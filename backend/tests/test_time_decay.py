"""Tests for time decay calculation."""

import pytest
import numpy as np
import pandas as pd
from datetime import datetime, timedelta
from app.rating.time_decay import TimeDecay


class TestTimeDecay:
    """Test suite for TimeDecay."""

    def test_initialization(self):
        """Test time decay initialization."""
        decay = TimeDecay(half_life_days=1095)

        assert decay.half_life_days == 1095
        assert decay.lambda_param == np.log(2) / 1095

    def test_calculate_weights_today(self):
        """Test weight for game played today."""
        decay = TimeDecay(half_life_days=1095)

        today = datetime.now()
        played_dates = pd.Series([today])

        weights = decay.calculate_weights(played_dates)

        # Today's game should have weight very close to 1.0
        assert len(weights) == 1
        assert abs(weights[0] - 1.0) < 0.01

    def test_calculate_weights_half_life(self):
        """Test weight at exactly half-life point."""
        decay = TimeDecay(half_life_days=1095)

        reference_date = datetime.now()
        half_life_ago = reference_date - timedelta(days=1095)
        played_dates = pd.Series([half_life_ago])

        weights = decay.calculate_weights(played_dates, reference_date=reference_date)

        # Game from half-life ago should have weight ~0.5
        assert len(weights) == 1
        assert abs(weights[0] - 0.5) < 0.01

    def test_calculate_weights_multiple_ages(self):
        """Test weights for games at various ages."""
        decay = TimeDecay(half_life_days=1095)  # 3 years

        reference_date = datetime(2025, 1, 1)
        played_dates = pd.Series([
            reference_date,  # Today
            reference_date - timedelta(days=365),  # 1 year ago
            reference_date - timedelta(days=1095),  # 3 years ago (half-life)
            reference_date - timedelta(days=2190),  # 6 years ago (2× half-life)
        ])

        weights = decay.calculate_weights(played_dates, reference_date=reference_date)

        assert len(weights) == 4

        # Today: ~1.0
        assert abs(weights[0] - 1.0) < 0.01

        # 1 year ago: ~0.71 (approximately)
        assert 0.68 < weights[1] < 0.74

        # 3 years ago: ~0.5 (half-life)
        assert abs(weights[2] - 0.5) < 0.01

        # 6 years ago: ~0.25 (quarter weight)
        assert abs(weights[3] - 0.25) < 0.01

    def test_weights_decrease_with_age(self):
        """Test that older games have lower weights."""
        decay = TimeDecay(half_life_days=1095)

        reference_date = datetime(2025, 1, 1)
        played_dates = pd.Series([
            reference_date - timedelta(days=100),
            reference_date - timedelta(days=200),
            reference_date - timedelta(days=300),
            reference_date - timedelta(days=400),
        ])

        weights = decay.calculate_weights(played_dates, reference_date=reference_date)

        # Weights should decrease monotonically
        assert weights[0] > weights[1]
        assert weights[1] > weights[2]
        assert weights[2] > weights[3]

    def test_future_games_handling(self):
        """Test that future games get weight 1.0."""
        decay = TimeDecay(half_life_days=1095)

        reference_date = datetime(2025, 1, 1)
        future_date = reference_date + timedelta(days=100)
        played_dates = pd.Series([future_date])

        weights = decay.calculate_weights(played_dates, reference_date=reference_date)

        # Future games should be clamped to weight 1.0
        assert weights[0] == 1.0

    def test_get_weight_for_age(self):
        """Test getting weight for specific age."""
        decay = TimeDecay(half_life_days=1095)

        # Today (0 days ago)
        assert abs(decay.get_weight_for_age(0) - 1.0) < 0.01

        # Half-life
        assert abs(decay.get_weight_for_age(1095) - 0.5) < 0.01

        # Double half-life
        assert abs(decay.get_weight_for_age(2190) - 0.25) < 0.01

        # Negative days should return 1.0
        assert abs(decay.get_weight_for_age(-100) - 1.0) < 0.01

    def test_effective_games_count(self):
        """Test effective games count calculation."""
        decay = TimeDecay(half_life_days=1095)

        # All games today → effective count = actual count
        weights = np.array([1.0, 1.0, 1.0, 1.0, 1.0])
        effective = decay.get_effective_games_count(weights)
        assert effective == 5.0

        # All games at half-life → effective count = half
        weights = np.array([0.5, 0.5, 0.5, 0.5])
        effective = decay.get_effective_games_count(weights)
        assert abs(effective - 2.0) < 0.01

        # Mixed ages
        weights = np.array([1.0, 0.8, 0.6, 0.4, 0.2])
        effective = decay.get_effective_games_count(weights)
        assert abs(effective - 3.0) < 0.01

    def test_get_decay_info(self):
        """Test decay information retrieval."""
        decay = TimeDecay(half_life_days=1095)

        info = decay.get_decay_info()

        assert info['half_life_days'] == 1095
        assert abs(info['half_life_years'] - 3.0) < 0.01
        assert 'example_weights' in info

        # Check example weights
        examples = info['example_weights']
        assert abs(examples['today'] - 1.0) < 0.01
        assert abs(examples['3_years'] - 0.5) < 0.01
        assert abs(examples['6_years'] - 0.25) < 0.01

    def test_different_half_lives(self):
        """Test with different half-life parameters."""
        decay_1yr = TimeDecay(half_life_days=365)
        decay_5yr = TimeDecay(half_life_days=1825)

        reference_date = datetime(2025, 1, 1)
        one_year_ago = reference_date - timedelta(days=365)
        played_dates = pd.Series([one_year_ago])

        # 1-year half-life: 1 year ago → weight 0.5
        weights_1yr = decay_1yr.calculate_weights(played_dates, reference_date=reference_date)
        assert abs(weights_1yr[0] - 0.5) < 0.01

        # 5-year half-life: 1 year ago → weight higher than 0.5
        weights_5yr = decay_5yr.calculate_weights(played_dates, reference_date=reference_date)
        assert weights_5yr[0] > 0.8

    def test_numpy_array_input(self):
        """Test with numpy array input."""
        decay = TimeDecay(half_life_days=1095)

        reference_date = datetime(2025, 1, 1)
        played_dates = np.array([
            reference_date - timedelta(days=100),
            reference_date - timedelta(days=200),
        ])

        weights = decay.calculate_weights(played_dates, reference_date=reference_date)

        assert len(weights) == 2
        assert all(0 < w <= 1.0 for w in weights)

    def test_single_game_weight(self):
        """Test weight calculation for a single game."""
        decay = TimeDecay(half_life_days=1095)

        reference_date = datetime(2025, 1, 1)
        played_dates = pd.Series([reference_date - timedelta(days=500)])

        weights = decay.calculate_weights(played_dates, reference_date=reference_date)

        assert len(weights) == 1
        assert 0.5 < weights[0] < 1.0  # Between half-life and today

    def test_weights_sum(self):
        """Test that weights are reasonable for large dataset."""
        decay = TimeDecay(half_life_days=1095)

        reference_date = datetime(2025, 1, 1)

        # Create 200 games spread over 4 years
        played_dates = []
        for days_ago in range(0, 1460, 7):  # Weekly games for 4 years
            played_dates.append(reference_date - timedelta(days=days_ago))

        played_dates = pd.Series(played_dates)
        weights = decay.calculate_weights(played_dates, reference_date=reference_date)

        # Sum of weights should be less than total games (due to decay)
        assert weights.sum() < len(weights)

        # But should be more than half (since many recent games)
        assert weights.sum() > len(weights) / 2
