use crate::models::{ConfidenceLevel, Game, Rating};
use anyhow::Result;
use chrono::Utc;
use ndarray::{Array1, Array2};
use std::collections::HashMap;
use tracing::info;

const STARTER_RATING: f64 = 500.0;
const CONVERGENCE_TOLERANCE: f64 = 1e-6;
const MAX_ITERATIONS: usize = 100;

/// Bradley-Terry Maximum Likelihood rating calculator
pub struct RatingCalculator {
    convergence_tolerance: f64,
    max_iterations: usize,
}

impl RatingCalculator {
    /// Create a new rating calculator
    pub fn new() -> Self {
        Self {
            convergence_tolerance: CONVERGENCE_TOLERANCE,
            max_iterations: MAX_ITERATIONS,
        }
    }

    /// Calculate ratings for all players using Bradley-Terry ML
    pub fn calculate_ratings(&self, games: &[Game]) -> Result<Vec<Rating>> {
        info!("Calculating ratings for {} games", games.len());

        // Build player index mapping
        let player_ids = self.extract_player_ids(games);
        let n_players = player_ids.len();
        info!("Found {} unique players", n_players);

        let player_to_idx: HashMap<i64, usize> = player_ids
            .iter()
            .enumerate()
            .map(|(idx, &id)| (id, idx))
            .collect();

        // Count games per player
        let mut games_count: HashMap<i64, i32> = HashMap::new();
        for game in games {
            *games_count.entry(game.player1_id).or_insert(0) += 1;
            *games_count.entry(game.player2_id).or_insert(0) += 1;
        }

        // Build comparison matrix and wins vector
        let (comparison_matrix, wins) = self.build_comparison_data(games, &player_to_idx);

        // Run MM (Minorization-Maximization) algorithm
        let log_ratings = self.mm_algorithm(&comparison_matrix, &wins, n_players);

        // Convert log ratings to actual ratings and create Rating structs
        let mut ratings = Vec::new();
        for (idx, &player_id) in player_ids.iter().enumerate() {
            let log_rating = log_ratings[idx];
            let rating_value = log_rating.exp() * STARTER_RATING;
            let games_played = *games_count.get(&player_id).unwrap_or(&0);

            ratings.push(Rating {
                player_id,
                rating: rating_value,
                games_played,
                confidence_level: Self::get_confidence_level(games_played),
                calculated_at: Utc::now(),
            });
        }

        info!("Rating calculation complete");
        Ok(ratings)
    }

    /// Extract unique player IDs from games
    fn extract_player_ids(&self, games: &[Game]) -> Vec<i64> {
        let mut player_ids: Vec<i64> = games
            .iter()
            .flat_map(|g| vec![g.player1_id, g.player2_id])
            .collect();

        player_ids.sort_unstable();
        player_ids.dedup();
        player_ids
    }

    /// Build comparison matrix and wins vector for Bradley-Terry
    fn build_comparison_data(
        &self,
        games: &[Game],
        player_to_idx: &HashMap<i64, usize>,
    ) -> (Array2<f64>, Array1<f64>) {
        let n_players = player_to_idx.len();
        let mut comparison_matrix = Array2::<f64>::zeros((n_players, n_players));
        let mut wins = Array1::<f64>::zeros(n_players);

        for game in games {
            let i = player_to_idx[&game.player1_id];
            let j = player_to_idx[&game.player2_id];
            let weight = game.weight;

            // Update comparison counts (how many times i and j played)
            comparison_matrix[[i, j]] += weight;
            comparison_matrix[[j, i]] += weight;

            // Update wins (who won this game)
            if game.player1_score > game.player2_score {
                wins[i] += weight;
            } else if game.player2_score > game.player1_score {
                wins[j] += weight;
            } else {
                // Tie - each gets half a win
                wins[i] += weight * 0.5;
                wins[j] += weight * 0.5;
            }
        }

        (comparison_matrix, wins)
    }

    /// MM (Minorization-Maximization) algorithm for Bradley-Terry ML estimation
    /// This is the Hunter (2004) algorithm - much faster than Newton methods for large datasets
    fn mm_algorithm(
        &self,
        comparison_matrix: &Array2<f64>,
        wins: &Array1<f64>,
        n_players: usize,
    ) -> Array1<f64> {
        // Initialize log-ratings to 0 (ratings = 1.0)
        let mut log_gamma = Array1::<f64>::zeros(n_players);

        for iteration in 0..self.max_iterations {
            let mut new_log_gamma = Array1::<f64>::zeros(n_players);

            // MM update step
            for i in 0..n_players {
                let mut denominator = 0.0;

                for j in 0..n_players {
                    if i != j && comparison_matrix[[i, j]] > 0.0 {
                        let gamma_i = log_gamma[i].exp();
                        let gamma_j = log_gamma[j].exp();
                        let comparisons = comparison_matrix[[i, j]];

                        denominator += comparisons / (gamma_i + gamma_j);
                    }
                }

                if denominator > 0.0 {
                    new_log_gamma[i] = (wins[i] / denominator).ln();
                }
            }

            // Normalize to prevent drift (keep mean at 0)
            let mean = new_log_gamma.mean().unwrap();
            new_log_gamma.mapv_inplace(|x| x - mean);

            // Check convergence
            let max_diff = (&new_log_gamma - &log_gamma)
                .mapv(|x| x.abs())
                .fold(0.0_f64, |a, &b| a.max(b));

            log_gamma = new_log_gamma;

            if max_diff < self.convergence_tolerance {
                info!("MM algorithm converged in {} iterations", iteration + 1);
                break;
            }
        }

        log_gamma
    }

    /// Determine confidence level based on games played
    fn get_confidence_level(games_played: i32) -> ConfidenceLevel {
        match games_played {
            0..=9 => ConfidenceLevel::Unranked,
            10..=49 => ConfidenceLevel::Provisional,
            50..=199 => ConfidenceLevel::Emerging,
            _ => ConfidenceLevel::Established,
        }
    }
}

impl Default for RatingCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_rating_calculation() {
        let calculator = RatingCalculator::new();

        // Create some test games
        let games = vec![
            Game {
                id: 1,
                tournament_id: 1,
                player1_id: 1,
                player2_id: 2,
                player1_score: 5,
                player2_score: 3,
                date: Utc::now(),
                weight: 1.0,
            },
            Game {
                id: 2,
                tournament_id: 1,
                player1_id: 2,
                player2_id: 3,
                player1_score: 5,
                player2_score: 2,
                date: Utc::now(),
                weight: 1.0,
            },
        ];

        let ratings = calculator.calculate_ratings(&games).unwrap();

        assert_eq!(ratings.len(), 3); // 3 unique players
        assert!(ratings.iter().all(|r| r.rating > 0.0));
    }
}
