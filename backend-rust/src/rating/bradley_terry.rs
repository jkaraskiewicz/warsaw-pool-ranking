use std::collections::HashMap;

use super::convergence::{has_converged, should_continue};
use super::normalization::normalize_ratings;
use super::types::{GameResult, PlayerId, PlayerRating, RatingMap};

const INITIAL_RATING: f64 = 1.0;

pub fn calculate_ratings(games: &[GameResult]) -> Vec<PlayerRating> {
    let mut ratings = initialize_ratings(games);
    let games_count = count_games_per_player(games);

    iterate_until_convergence(&mut ratings, games);
    normalize_ratings(&mut ratings);

    build_player_ratings(ratings, games_count)
}

fn initialize_ratings(games: &[GameResult]) -> RatingMap {
    let mut ratings = RatingMap::new();

    for game in games {
        ratings.entry(game.winner_id).or_insert(INITIAL_RATING);
        ratings.entry(game.loser_id).or_insert(INITIAL_RATING);
    }

    ratings
}

fn count_games_per_player(games: &[GameResult]) -> HashMap<PlayerId, i32> {
    let mut counts = HashMap::new();

    for game in games {
        *counts.entry(game.winner_id).or_insert(0) += 1;
        *counts.entry(game.loser_id).or_insert(0) += 1;
    }

    counts
}

fn iterate_until_convergence(ratings: &mut RatingMap, games: &[GameResult]) {
    let mut iteration = 0;

    while should_continue(iteration) {
        let new_ratings = compute_iteration(ratings, games);

        if has_converged(ratings, &new_ratings) {
            break;
        }

        *ratings = new_ratings;
        iteration += 1;
    }
}

fn compute_iteration(
    current_ratings: &RatingMap,
    games: &[GameResult],
) -> RatingMap {
    let mut new_ratings = RatingMap::new();

    for (&player_id, _) in current_ratings.iter() {
        let new_rating = calculate_player_rating(
            player_id,
            current_ratings,
            games,
        );
        new_ratings.insert(player_id, new_rating);
    }

    new_ratings
}

fn calculate_player_rating(
    player_id: PlayerId,
    ratings: &RatingMap,
    games: &[GameResult],
) -> f64 {
    let wins = count_weighted_wins(player_id, games);
    let denominators = sum_denominators(player_id, ratings, games);

    if denominators > 0.0 {
        wins / denominators
    } else {
        INITIAL_RATING
    }
}

fn count_weighted_wins(player_id: PlayerId, games: &[GameResult]) -> f64 {
    games
        .iter()
        .filter(|g| g.winner_id == player_id)
        .map(|g| g.weight)
        .sum()
}

fn sum_denominators(
    player_id: PlayerId,
    ratings: &RatingMap,
    games: &[GameResult],
) -> f64 {
    games
        .iter()
        .filter(|g| involves_player(g, player_id))
        .map(|g| calculate_denominator(g, player_id, ratings))
        .sum()
}

fn involves_player(game: &GameResult, player_id: PlayerId) -> bool {
    game.winner_id == player_id || game.loser_id == player_id
}

fn calculate_denominator(
    game: &GameResult,
    player_id: PlayerId,
    ratings: &RatingMap,
) -> f64 {
    let opponent_id = get_opponent_id(game, player_id);
    let player_rating = get_rating(ratings, player_id);
    let opponent_rating = get_rating(ratings, opponent_id);

    game.weight / (player_rating + opponent_rating)
}

fn get_opponent_id(game: &GameResult, player_id: PlayerId) -> PlayerId {
    if game.winner_id == player_id {
        game.loser_id
    } else {
        game.winner_id
    }
}

fn get_rating(ratings: &RatingMap, player_id: PlayerId) -> f64 {
    ratings.get(&player_id).copied().unwrap_or(INITIAL_RATING)
}

fn build_player_ratings(
    ratings: RatingMap,
    games_count: HashMap<PlayerId, i32>,
) -> Vec<PlayerRating> {
    ratings
        .into_iter()
        .map(|(id, rating)| build_single_rating(id, rating, &games_count))
        .collect()
}

fn build_single_rating(
    player_id: PlayerId,
    rating: f64,
    games_count: &HashMap<PlayerId, i32>,
) -> PlayerRating {
    use super::types::ConfidenceLevel;

    let games_played = games_count.get(&player_id).copied().unwrap_or(0);
    let confidence_level = ConfidenceLevel::from_games_played(games_played);

    PlayerRating {
        player_id,
        rating,
        games_played,
        confidence_level,
    }
}
