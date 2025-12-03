pub mod api;
pub mod cache;
pub mod cli;
pub mod config;
pub mod database;
pub mod domain;
pub mod errors;
pub mod fetchers;
pub mod http;
pub mod pagination;
pub mod rate_limiter;
pub mod rating;

use std::collections::HashSet;

use anyhow::{Context, Result};
use clap::Parser;
use cli::Cli;

use crate::api::CueScoreClient;
use crate::cache::Cache;
use crate::cli::Command;
use crate::domain::{FetchProgress, TournamentCollection};
use crate::fetchers::VenueScraper;

pub fn interpret() -> Command {
    let cli = Cli::parse();
    cli.command
}

pub fn handle_serve(_port: u16) -> Result<()> {
    todo!()
}

pub fn handle_ingest() -> Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(ingest_data())
}

async fn ingest_data() -> Result<()> {
    use log::info;

    info!("=== Starting Data Ingestion ===\n");

    let cache = Cache::new("cache")?;
    let mut scraper = VenueScraper::new()?;
    let mut api_client = CueScoreClient::new()?;

    // Step 1: Discover tournaments
    let tournament_ids = discover_tournaments(&mut scraper).await?;
    info!("  → Found {} unique tournaments\n", tournament_ids.len());

    // Step 2: Fetch tournament data
    let collection = fetch_tournaments(&mut api_client, &cache, tournament_ids).await?;
    info!("  → Fetched {} tournaments with data\n", collection.len());

    // Step 3: Save to parsed cache
    save_parsed_cache(&cache, collection)?;
    info!("  → Saved to parsed cache\n");

    info!("=== Ingestion Complete ===");
    Ok(())
}

async fn discover_tournaments(scraper: &mut VenueScraper) -> Result<HashSet<i64>> {
    use log::info;
    use crate::config::get_venues;

    info!("Step 1: Discovering tournaments from venues...");

    let venues = get_venues();
    let mut all_ids = HashSet::new();

    for venue in venues {
        let ids = scraper.scrape_venue_tournaments(venue.id, venue.name, None).await?;
        all_ids.extend(ids);
    }

    Ok(all_ids)
}

async fn fetch_tournaments(
    client: &mut CueScoreClient,
    cache: &Cache,
    tournament_ids: HashSet<i64>,
) -> Result<TournamentCollection> {
    use log::info;

    info!("Step 2: Fetching tournament details...");

    let total = tournament_ids.len();
    let mut progress = FetchProgress::new(total);
    let mut collection = TournamentCollection::new();

    for tournament_id in tournament_ids {
        let was_cached = is_cached(cache, tournament_id)?;

        let tournament = fetch_single_tournament(client, cache, tournament_id).await?;
        collection.add(tournament);

        update_progress(&mut progress, was_cached);
    }

    Ok(collection)
}

fn is_cached(cache: &Cache, tournament_id: i64) -> Result<bool> {
    Ok(cache.load_raw(&tournament_id.to_string())?.is_some())
}

fn update_progress(progress: &mut FetchProgress, was_cached: bool) {
    if was_cached {
        progress.increment_cached();
    } else {
        progress.increment_fetched();
    }
}

async fn fetch_single_tournament(
    client: &mut CueScoreClient,
    cache: &Cache,
    tournament_id: i64,
) -> Result<crate::domain::TournamentResponse> {
    client.fetch_and_cache_tournament(tournament_id, cache).await
}

fn save_parsed_cache(cache: &Cache, collection: TournamentCollection) -> Result<()> {
    use log::info;

    info!("Step 3: Saving parsed tournament cache...");

    let tournaments = collection.into_vec();
    cache.save_parsed("tournaments", &tournaments)?;

    Ok(())
}

pub fn handle_process() -> Result<()> {
    use log::info;

    info!("=== Starting Data Processing ===\n");

    let cache = Cache::new("cache")?;
    let db_path = std::env::var("DATABASE_PATH")
        .unwrap_or_else(|_| "warsaw_pool_ranking.db".to_string());

    let pool = database::create_pool(&db_path)?;
    let mut conn = database::get_connection(&pool)?;

    // Step 1: Reset database (PoC - no migrations)
    database::setup::reset_database(&mut conn)?;
    info!("  → Database schema reset\n");

    // Step 2: Load cached tournaments
    let tournaments = load_tournaments_from_cache(&cache)?;
    info!("  → Loaded {} tournaments from cache\n", tournaments.len());

    // Step 3: Insert tournaments and expand to games
    let expanded_games = process_tournaments(&mut conn, &tournaments)?;
    info!("  → Expanded to {} individual games\n", expanded_games.len());

    // Step 4: Calculate ratings
    let ratings = calculate_player_ratings(&expanded_games)?;
    info!("  → Calculated ratings for {} players\n", ratings.len());

    // Step 5: Save ratings to database
    save_ratings_to_db(&mut conn, &ratings)?;
    info!("  → Saved ratings to database\n");

    info!("=== Processing Complete ===");
    Ok(())
}

fn load_tournaments_from_cache(
    cache: &Cache,
) -> Result<Vec<crate::domain::TournamentResponse>> {
    cache
        .load_parsed("tournaments")?
        .ok_or_else(|| anyhow::anyhow!("No tournaments found in cache"))
}

fn process_tournaments(
    conn: &mut database::DbConn,
    tournaments: &[crate::domain::TournamentResponse],
) -> Result<Vec<domain::ExpandedGame>> {
    use log::info;
    use std::collections::HashMap;

    let mut all_games = Vec::new();

    for (idx, tournament) in tournaments.iter().enumerate() {
        if (idx + 1) % 10 == 0 || idx + 1 == tournaments.len() {
            info!("  Processing tournament {}/{}", idx + 1, tournaments.len());
        }

        // Extract player names from tournament
        let player_names = extract_player_names(tournament);

        let tournament_db = insert_tournament_to_db(conn, tournament)?;
        let mut games = expand_tournament_games(tournament)?;

        insert_games_to_db(conn, &games, tournament_db.id, &player_names)?;
        all_games.append(&mut games);
    }

    apply_time_decay_weights(&mut all_games);
    Ok(all_games)
}

fn insert_tournament_to_db(
    conn: &mut database::DbConn,
    tournament: &crate::domain::TournamentResponse,
) -> Result<database::Tournament> {
    let start_date = parse_tournament_date(&tournament.starttime)?;
    let end_date = parse_optional_tournament_date(&tournament.stoptime)?;

    database::tournaments::upsert_tournament(
        conn,
        tournament.id,
        &tournament.name,
        tournament.venue_id(),
        &tournament.venue_name(),
        start_date,
        end_date,
    )
}

fn parse_tournament_date(date_str: &str) -> Result<chrono::NaiveDateTime> {
    use chrono::{DateTime, NaiveDateTime as ND};

    // Try RFC3339 format (with timezone)
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.naive_utc());
    }

    // Try naive datetime format (without timezone)
    if let Ok(dt) = ND::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt);
    }

    // Try with fractional seconds
    if let Ok(dt) = ND::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(dt);
    }

    anyhow::bail!("Failed to parse tournament date: {}", date_str)
}

fn parse_optional_tournament_date(
    date_str: &Option<String>,
) -> Result<Option<chrono::NaiveDateTime>> {
    match date_str {
        Some(s) => Ok(Some(parse_tournament_date(s)?)),
        None => Ok(None),
    }
}

fn expand_tournament_games(
    tournament: &crate::domain::TournamentResponse,
) -> Result<Vec<domain::ExpandedGame>> {
    domain::games_expansion::expand_tournament_to_games(tournament)
}

fn extract_player_names(
    tournament: &crate::domain::TournamentResponse,
) -> std::collections::HashMap<i64, String> {
    let mut names = std::collections::HashMap::new();

    for match_data in &tournament.matches {
        if match_data.is_played() {
            names.insert(match_data.player_a_id(), match_data.player_a_name());
            names.insert(match_data.player_b_id(), match_data.player_b_name());
        }
    }

    names
}

fn insert_games_to_db(
    conn: &mut database::DbConn,
    games: &[domain::ExpandedGame],
    tournament_db_id: i32,
    player_names: &std::collections::HashMap<i64, String>,
) -> Result<()> {
    for game in games {
        let first_player = upsert_player(conn, game.winner_id, player_names)?;
        let second_player = upsert_player(conn, game.loser_id, player_names)?;

        database::games::insert_game(
            conn,
            tournament_db_id,
            first_player.id,
            second_player.id,
            1, // winner scored 1
            0, // loser scored 0
            game.date,
            game.weight,
        )?;
    }

    Ok(())
}

fn upsert_player(
    conn: &mut database::DbConn,
    cuescore_id: i64,
    player_names: &std::collections::HashMap<i64, String>,
) -> Result<database::Player> {
    let name = player_names
        .get(&cuescore_id)
        .map(|s| s.as_str())
        .unwrap_or("Unknown Player");
    database::players::upsert_player(conn, cuescore_id, name)
}

fn apply_time_decay_weights(games: &mut [domain::ExpandedGame]) {
    let current_date = chrono::Utc::now().naive_utc();
    rating::weighting::apply_weights_to_games(games, current_date);
}

fn calculate_player_ratings(
    games: &[domain::ExpandedGame],
) -> Result<Vec<rating::PlayerRating>> {
    let game_results = convert_to_game_results(games);
    let ratings = rating::calculate_ratings(&game_results);
    Ok(ratings)
}

fn convert_to_game_results(
    games: &[domain::ExpandedGame],
) -> Vec<rating::GameResult> {
    games
        .iter()
        .map(|g| rating::GameResult {
            winner_id: g.winner_id as i32,
            loser_id: g.loser_id as i32,
            weight: g.weight,
        })
        .collect()
}

fn save_ratings_to_db(
    conn: &mut database::DbConn,
    ratings: &[rating::PlayerRating],
) -> Result<()> {
    let calculated_at = chrono::Utc::now().naive_utc();

    for player_rating in ratings {
        // player_rating.player_id is actually a cuescore_id (i64)
        // We need to look up the actual database player_id (i32)
        let cuescore_id = player_rating.player_id as i64;
        let player = database::players::upsert_player(conn, cuescore_id, "Unknown Player")?;

        database::ratings::insert_rating(
            conn,
            player.id,
            player_rating.rating,
            player_rating.games_played,
            player_rating.confidence_level.as_str(),
            calculated_at,
        )?;
    }

    Ok(())
}
