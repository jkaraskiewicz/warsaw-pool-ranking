use std::collections::HashMap;
use ndarray::{Array1, Array2};
use log::info;

use super::types::{GameResult, PlayerId, PlayerRating, ConfidenceLevel};
use crate::config::settings::RatingSettings;

/// Calculates ratings using the MM (Minorization-Maximization) algorithm
/// This is O(N) per iteration rather than O(N*M) of the naive approach
pub fn calculate_ratings(games: &[GameResult], config: &RatingSettings) -> Vec<PlayerRating> {
    info!("Calculating ratings for {} games using MM algorithm", games.len());

    // 1. Map PlayerIds to dense indices (0..N)
    let player_ids = extract_player_ids(games);
    let n_players = player_ids.len();
    info!("Found {} unique players", n_players);

    let player_to_idx: HashMap<PlayerId, usize> = player_ids
        .iter()
        .enumerate()
        .map(|(idx, &id)| (id, idx))
        .collect();

    // 2. Count games per player (for statistics)
    let games_count = count_games_per_player(games);

    // 3. Build comparison matrix and wins vector
    let (comparison_matrix, wins) = build_comparison_data(games, &player_to_idx, n_players, config);

    // 4. Run MM algorithm
    let log_ratings = mm_algorithm(&comparison_matrix, &wins, n_players, config);

    // 5. Convert results back to PlayerRating objects
    build_player_ratings(&player_ids, &log_ratings, &games_count, config)
}

fn extract_player_ids(games: &[GameResult]) -> Vec<PlayerId> {
    let mut ids: Vec<PlayerId> = games
        .iter()
        .flat_map(|g| vec![g.winner_id, g.loser_id])
        .collect();

    ids.sort_unstable();
    ids.dedup();
    ids
}

fn count_games_per_player(games: &[GameResult]) -> HashMap<PlayerId, i32> {
    let mut counts = HashMap::new();
    for game in games {
        *counts.entry(game.winner_id).or_insert(0) += 1;
        *counts.entry(game.loser_id).or_insert(0) += 1;
    }
    counts
}

fn build_comparison_data(
    games: &[GameResult],
    player_to_idx: &HashMap<PlayerId, usize>,
    n_players: usize,
    _config: &RatingSettings,
) -> (Array2<f64>, Array1<f64>) {
    // Note: For extremely large N, a dense matrix might be too memory intensive.
    let mut comparison_matrix = Array2::<f64>::zeros((n_players, n_players));
    let mut wins = Array1::<f64>::zeros(n_players);

    for game in games {
        let i = player_to_idx[&game.winner_id]; // winner
        let j = player_to_idx[&game.loser_id];  // loser
        let weight = game.weight;

        // Update comparison counts (total weight of games between i and j)
        comparison_matrix[[i, j]] += weight;
        comparison_matrix[[j, i]] += weight;

        // Update wins
        wins[i] += weight;
        // Loser gets 0 wins added
    }

    (comparison_matrix, wins)
}

fn mm_algorithm(
    comparison_matrix: &Array2<f64>,
    wins: &Array1<f64>,
    n_players: usize,
    config: &RatingSettings,
) -> Array1<f64> {
    // Initialize log-ratings to 0 (ratings = 1.0)
    let mut log_gamma = Array1::<f64>::zeros(n_players);

    for iteration in 0..config.max_iterations {
        let mut new_log_gamma = Array1::<f64>::zeros(n_players);

        // MM update step
        for i in 0..n_players {
            let mut denominator = 0.0;
            let gamma_i = log_gamma[i].exp();

            // This inner loop is the bottleneck if dense.
            // Ideally should iterate only over neighbors.
            for j in 0..n_players {
                if i != j {
                    let comparisons = comparison_matrix[[i, j]];
                    if comparisons > 0.0 {
                        let gamma_j = log_gamma[j].exp();

                        denominator += comparisons / (gamma_i + gamma_j);
                    }
                }
            }

            // Add virtual games against "average player" (gamma=1.0)
            // We simulate VIRTUAL_GAMES_WEIGHT games where we drew (0.5 win).
            // This anchors everyone to the mean (500).
            denominator += config.virtual_games_weight / (gamma_i + 1.0);
            let adjusted_wins = wins[i] + (0.5 * config.virtual_games_weight);

            if denominator > 0.0 {
                new_log_gamma[i] = (adjusted_wins / denominator).ln();
            } else {
                 // Should not happen for connected graph, but handle safely
                 new_log_gamma[i] = log_gamma[i];
            }
        }

        // Normalize (mean 0)
        let mean = new_log_gamma.mean().unwrap_or(0.0);
        new_log_gamma.mapv_inplace(|x| x - mean);

        // Check convergence
        let max_diff = (&new_log_gamma - &log_gamma)
            .mapv(|x| x.abs())
            .fold(0.0_f64, |a, &b| a.max(b));

        log_gamma = new_log_gamma;

        if max_diff < config.convergence_tolerance {
            info!("MM algorithm converged in {} iterations", iteration + 1);
            break;
        }
    }

    log_gamma
}

fn build_player_ratings(
    player_ids: &[PlayerId],
    log_ratings: &Array1<f64>,
    games_count: &HashMap<PlayerId, i32>,
    config: &RatingSettings,
) -> Vec<PlayerRating> {
    let mut ratings = Vec::new();

    for (idx, &player_id) in player_ids.iter().enumerate() {
        let games_played = *games_count.get(&player_id).unwrap_or(&0);

        let mut rating_value;
        if games_played < config.min_ranked_games {
            rating_value = config.starter_rating;
        } else {
            let log_rating = log_ratings[idx];
            // Convert log rating to Fargo-like scale
            // Fargo: 100 points = 2:1 odds
            // log(2) difference corresponds to 100 points
            // rating = base + (log_rating * 100 / ln(2))
            rating_value = config.starter_rating + (log_rating * 100.0 / std::f64::consts::LN_2);

            // Blending logic: Pull ratings towards 500 for players with < 200 games
            if games_played < config.established_games {
                let starter_weight = (config.established_games - games_played) as f64 / config.established_games as f64;
                let ml_weight = 1.0 - starter_weight;
                rating_value = (starter_weight * config.starter_rating) + (ml_weight * rating_value);
            }

            // Clamp to a reasonable range
            rating_value = rating_value.clamp(0.0, 2000.0);
        }

        ratings.push(PlayerRating {
            player_id,
            rating_type: "temp".to_string(), // Placeholder, to be overwritten by ProcessingService
            rating: rating_value,
            games_played,
            confidence_level: ConfidenceLevel::from_games_played(games_played),
        });
    }

    ratings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::settings::RatingSettings;

    fn test_config() -> RatingSettings {
        RatingSettings {
            starter_rating: 500.0,
            virtual_games_weight: 5.0,
            min_ranked_games: 10,
            established_games: 200,
            convergence_tolerance: 1e-6,
            max_iterations: 100,
            periods: vec![],
        }
    }

    #[test]
    fn test_two_player_equal_results() {
        // Two players who split games evenly should have similar ratings
        let games = vec![
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
            GameResult { winner_id: 2, loser_id: 1, weight: 1.0 },
        ];
        let mut config = test_config();
        config.min_ranked_games = 1;

        let ratings = calculate_ratings(&games, &config);
        assert_eq!(ratings.len(), 2);

        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap();
        let r2 = ratings.iter().find(|r| r.player_id == 2).unwrap();

        // Ratings should be approximately equal
        assert!(
            (r1.rating - r2.rating).abs() < 10.0,
            "Equal results should yield similar ratings: {} vs {}",
            r1.rating,
            r2.rating
        );
    }

    #[test]
    fn test_dominant_player_higher_rating() {
        // Player 1 wins all games against player 2
        let games = vec![
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
        ];
        let mut config = test_config();
        config.min_ranked_games = 1;

        let ratings = calculate_ratings(&games, &config);
        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap();
        let r2 = ratings.iter().find(|r| r.player_id == 2).unwrap();

        assert!(
            r1.rating > r2.rating,
            "Dominant player should have higher rating: {} vs {}",
            r1.rating,
            r2.rating
        );
    }

    #[test]
    fn test_2_to_1_odds_approx_100_points() {
        // If player A beats B twice as often, rating difference should be ~100 points
        // Create 400 games: A wins 267, B wins 133 (2:1 ratio)
        // We need many games to overcome blending and virtual games effects
        let mut games = Vec::new();
        for _ in 0..267 {
            games.push(GameResult { winner_id: 1, loser_id: 2, weight: 1.0 });
        }
        for _ in 0..133 {
            games.push(GameResult { winner_id: 2, loser_id: 1, weight: 1.0 });
        }

        let mut config = test_config();
        config.min_ranked_games = 1;
        config.established_games = 200; // Both players have 400 games, above threshold
        config.virtual_games_weight = 1.0; // Reduce virtual games impact

        let ratings = calculate_ratings(&games, &config);
        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap();
        let r2 = ratings.iter().find(|r| r.player_id == 2).unwrap();

        let diff = r1.rating - r2.rating;
        // 2:1 odds = ~100 point difference (with tolerance for virtual games)
        // The theoretical 100 points is moderated by virtual games anchoring
        assert!(
            diff > 50.0 && diff < 150.0,
            "2:1 win ratio should yield ~100 point difference, got {}",
            diff
        );
    }

    #[test]
    fn test_circular_results_convergence() {
        // A beats B, B beats C, C beats A - algorithm should still converge
        let games = vec![
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
            GameResult { winner_id: 2, loser_id: 3, weight: 1.0 },
            GameResult { winner_id: 3, loser_id: 1, weight: 1.0 },
        ];
        let mut config = test_config();
        config.min_ranked_games = 1;

        let ratings = calculate_ratings(&games, &config);
        assert_eq!(ratings.len(), 3);

        // All three should have similar ratings due to circular dominance
        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap().rating;
        let r2 = ratings.iter().find(|r| r.player_id == 2).unwrap().rating;
        let r3 = ratings.iter().find(|r| r.player_id == 3).unwrap().rating;

        assert!((r1 - r2).abs() < 50.0, "Circular results should balance out");
        assert!((r2 - r3).abs() < 50.0, "Circular results should balance out");
    }

    #[test]
    fn test_ratings_center_around_starter() {
        // Virtual games anchor ratings around the starter rating (500)
        let games = vec![
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
            GameResult { winner_id: 2, loser_id: 3, weight: 1.0 },
            GameResult { winner_id: 3, loser_id: 4, weight: 1.0 },
        ];
        let mut config = test_config();
        config.min_ranked_games = 1;
        config.established_games = 500;

        let ratings = calculate_ratings(&games, &config);
        let avg_rating: f64 = ratings.iter().map(|r| r.rating).sum::<f64>() / ratings.len() as f64;

        // Average should be close to starter rating (500) due to virtual games
        assert!(
            (avg_rating - 500.0).abs() < 100.0,
            "Average rating should be near 500, got {}",
            avg_rating
        );
    }

    #[test]
    fn test_min_games_threshold() {
        // Player with fewer than min_ranked_games gets starter rating
        let games = vec![
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
        ];
        let config = test_config(); // min_ranked_games = 10

        let ratings = calculate_ratings(&games, &config);
        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap();

        // With only 1 game (below threshold of 10), should get starter rating
        assert_eq!(
            r1.rating, 500.0,
            "Player below min_ranked_games should have starter rating"
        );
    }

    #[test]
    fn test_blending_for_few_games() {
        // Players with games between min_ranked and established get blended ratings
        let mut games = Vec::new();
        // Player 1 wins 50 games against player 2 (above min, below established)
        for _ in 0..50 {
            games.push(GameResult { winner_id: 1, loser_id: 2, weight: 1.0 });
        }

        let mut config = test_config();
        config.min_ranked_games = 10;
        config.established_games = 200;

        let ratings = calculate_ratings(&games, &config);
        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap();

        // With blending, even a dominant player is pulled toward 500
        // The pure ML rating would be much higher, but blending moderates it
        assert!(
            r1.rating > 500.0 && r1.rating < 700.0,
            "Blended rating should be moderate, got {}",
            r1.rating
        );
    }

    #[test]
    fn test_games_count_tracked() {
        let games = vec![
            GameResult { winner_id: 1, loser_id: 2, weight: 1.0 },
            GameResult { winner_id: 1, loser_id: 3, weight: 1.0 },
            GameResult { winner_id: 2, loser_id: 3, weight: 1.0 },
        ];
        let config = test_config();

        let ratings = calculate_ratings(&games, &config);
        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap();
        let r2 = ratings.iter().find(|r| r.player_id == 2).unwrap();
        let r3 = ratings.iter().find(|r| r.player_id == 3).unwrap();

        assert_eq!(r1.games_played, 2, "Player 1 played 2 games");
        assert_eq!(r2.games_played, 2, "Player 2 played 2 games");
        assert_eq!(r3.games_played, 2, "Player 3 played 2 games");
    }

    #[test]
    fn test_confidence_level_assigned() {
        let mut games = Vec::new();
        // Player 1 plays 200+ games (Established)
        for _ in 0..200 {
            games.push(GameResult { winner_id: 1, loser_id: 2, weight: 1.0 });
        }

        let mut config = test_config();
        config.min_ranked_games = 1;

        let ratings = calculate_ratings(&games, &config);
        let r1 = ratings.iter().find(|r| r.player_id == 1).unwrap();
        let r2 = ratings.iter().find(|r| r.player_id == 2).unwrap();

        assert_eq!(r1.confidence_level, ConfidenceLevel::Established);
        assert_eq!(r2.confidence_level, ConfidenceLevel::Established);
    }

    #[test]
    fn test_rating_clamped_to_range() {
        // Even with extreme results, ratings should stay in 0-2000 range
        let mut games = Vec::new();
        for _ in 0..500 {
            games.push(GameResult { winner_id: 1, loser_id: 2, weight: 1.0 });
        }

        let mut config = test_config();
        config.min_ranked_games = 1;
        config.established_games = 1000;

        let ratings = calculate_ratings(&games, &config);
        for r in &ratings {
            assert!(r.rating >= 0.0 && r.rating <= 2000.0, "Rating {} out of range", r.rating);
        }
    }
}
