"""Tests for Bradley-Terry ML rating calculator."""

import pytest
import pandas as pd
import numpy as np
from app.rating.calculator import RatingCalculator


class TestRatingCalculator:
    """Test suite for RatingCalculator."""

    def test_initialization(self):
        """Test calculator initialization."""
        calc = RatingCalculator(base_rating=600)
        assert calc.base_rating == 600
        assert calc.scale_factor == 100 / np.log(2)

    def test_empty_games_df(self):
        """Test with empty games DataFrame."""
        calc = RatingCalculator()
        games_df = pd.DataFrame()

        ratings = calc.calculate_ratings(games_df)

        assert ratings == {}

    def test_missing_columns(self):
        """Test with missing required columns."""
        calc = RatingCalculator()
        games_df = pd.DataFrame({
            'player_a_id': [1],
            'winner_id': [1]
            # Missing player_b_id
        })

        with pytest.raises(ValueError, match="Missing required columns"):
            calc.calculate_ratings(games_df)

    def test_single_game(self):
        """Test rating calculation with a single game."""
        calc = RatingCalculator()

        # Player 1 beats Player 2 once
        games_df = pd.DataFrame({
            'player_a_id': [1],
            'player_b_id': [2],
            'winner_id': [1]
        })

        ratings = calc.calculate_ratings(games_df)

        # Both players should have ratings
        assert 1 in ratings
        assert 2 in ratings

        # Winner should have higher rating
        assert ratings[1] > ratings[2]

        # Ratings should be centered around base rating (500)
        mean_rating = (ratings[1] + ratings[2]) / 2
        assert abs(mean_rating - 500) < 1  # Close to 500

    def test_multiple_games_consistent_winner(self):
        """Test with multiple games where one player always wins."""
        calc = RatingCalculator()

        # Player 1 beats Player 2 ten times
        games_df = pd.DataFrame({
            'player_a_id': [1] * 10,
            'player_b_id': [2] * 10,
            'winner_id': [1] * 10
        })

        ratings = calc.calculate_ratings(games_df)

        # Player 1 should have significantly higher rating
        assert ratings[1] > ratings[2]
        assert ratings[1] - ratings[2] > 100  # At least 100 points difference

    def test_three_players_transitive(self):
        """Test with three players: A > B > C."""
        calc = RatingCalculator()

        games = []
        # Player 1 beats Player 2 (5 games)
        for _ in range(5):
            games.append({'player_a_id': 1, 'player_b_id': 2, 'winner_id': 1})

        # Player 2 beats Player 3 (5 games)
        for _ in range(5):
            games.append({'player_a_id': 2, 'player_b_id': 3, 'winner_id': 2})

        games_df = pd.DataFrame(games)
        ratings = calc.calculate_ratings(games_df)

        # Ratings should be transitive: 1 > 2 > 3
        assert ratings[1] > ratings[2]
        assert ratings[2] > ratings[3]

    def test_even_matchup(self):
        """Test with evenly matched players."""
        calc = RatingCalculator()

        games = []
        # Players alternate winning (10 games each)
        for i in range(10):
            games.append({'player_a_id': 1, 'player_b_id': 2, 'winner_id': 1})
            games.append({'player_a_id': 1, 'player_b_id': 2, 'winner_id': 2})

        games_df = pd.DataFrame(games)
        ratings = calc.calculate_ratings(games_df)

        # Ratings should be very close (within 10 points)
        assert abs(ratings[1] - ratings[2]) < 10

    def test_with_time_weights(self):
        """Test rating calculation with time decay weights."""
        calc = RatingCalculator()

        # Player 1 beats Player 2 five times
        games_df = pd.DataFrame({
            'player_a_id': [1] * 5,
            'player_b_id': [2] * 5,
            'winner_id': [1] * 5
        })

        # Apply different weights (older games weighted less)
        weights = np.array([0.5, 0.6, 0.7, 0.8, 1.0])

        ratings = calc.calculate_ratings(games_df, time_weights=weights)

        # Should still have both players
        assert 1 in ratings
        assert 2 in ratings

        # Player 1 should still win
        assert ratings[1] > ratings[2]

    def test_predict_win_probability(self):
        """Test win probability prediction."""
        calc = RatingCalculator()

        # Equal ratings → 50% probability
        prob = calc.predict_win_probability(500, 500)
        assert abs(prob - 0.5) < 0.01

        # 100-point advantage → ~67% probability (2:1 odds)
        prob = calc.predict_win_probability(600, 500)
        assert abs(prob - 0.667) < 0.01

        # 100-point disadvantage → ~33% probability
        prob = calc.predict_win_probability(500, 600)
        assert abs(prob - 0.333) < 0.01

        # 200-point advantage → ~80% probability (4:1 odds)
        prob = calc.predict_win_probability(700, 500)
        assert abs(prob - 0.8) < 0.01

    def test_rating_scale_calibration(self):
        """Test that 100 points = 2:1 odds relationship holds."""
        calc = RatingCalculator()

        # Create games where Player 1 wins 2/3 of the time (2:1 ratio)
        games = []
        for _ in range(20):  # Player 1 wins
            games.append({'player_a_id': 1, 'player_b_id': 2, 'winner_id': 1})
        for _ in range(10):  # Player 2 wins
            games.append({'player_a_id': 1, 'player_b_id': 2, 'winner_id': 2})

        games_df = pd.DataFrame(games)
        ratings = calc.calculate_ratings(games_df)

        rating_diff = ratings[1] - ratings[2]

        # Should be approximately 100 points difference
        # (might not be exactly 100 due to small sample size)
        assert 70 < rating_diff < 130

    def test_deterministic_results(self):
        """Test that same input produces same results."""
        calc = RatingCalculator()

        games_df = pd.DataFrame({
            'player_a_id': [1, 2, 3, 1, 2],
            'player_b_id': [2, 3, 1, 3, 1],
            'winner_id': [1, 2, 3, 1, 2]
        })

        ratings1 = calc.calculate_ratings(games_df)
        ratings2 = calc.calculate_ratings(games_df)

        # Results should be identical
        for player_id in ratings1:
            assert abs(ratings1[player_id] - ratings2[player_id]) < 0.001

    def test_large_dataset(self):
        """Test with a larger dataset."""
        calc = RatingCalculator()

        # Create 100 players
        games = []
        for i in range(1, 51):
            for j in range(i+1, 52):
                # Higher-numbered player has advantage
                winner = j if np.random.random() < 0.6 else i
                games.append({
                    'player_a_id': i,
                    'player_b_id': j,
                    'winner_id': winner
                })

        games_df = pd.DataFrame(games)
        ratings = calc.calculate_ratings(games_df)

        # Should have ratings for many players
        assert len(ratings) == 51

        # Ratings should generally increase with player number
        # (since higher numbers have advantage in our simulation)
        avg_rating_low = np.mean([ratings[i] for i in range(1, 26)])
        avg_rating_high = np.mean([ratings[i] for i in range(26, 51)])
        assert avg_rating_high > avg_rating_low
