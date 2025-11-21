"""Time decay calculation for weighting games by recency."""

import numpy as np
import pandas as pd
from datetime import datetime, timedelta
from typing import Union
import logging

logger = logging.getLogger(__name__)


class TimeDecay:
    """
    Calculates exponential time decay weights for games.

    Older games receive progressively less weight in rating calculations
    to account for player improvement/decline over time.
    """

    def __init__(self, half_life_days: int = 1095):
        """
        Initialize time decay calculator.

        Args:
            half_life_days: Number of days for weight to decay to 50%
                            Default: 1095 days (3 years)
        """
        self.half_life_days = half_life_days
        # Calculate decay constant: λ = ln(2) / half_life
        self.lambda_param = np.log(2) / half_life_days
        logger.info(f"TimeDecay initialized with {half_life_days}-day half-life (λ={self.lambda_param:.6f})")

    def calculate_weights(
        self,
        played_dates: Union[pd.Series, np.ndarray],
        reference_date: datetime = None
    ) -> np.ndarray:
        """
        Calculate exponential decay weights for games.

        Weight formula: w = exp(-λ × days_ago)
        where λ = ln(2) / half_life_days

        Args:
            played_dates: Series or array of datetime objects when games were played
            reference_date: The "now" date to calculate age from.
                           If None, uses current datetime.
                           For historical simulation, pass the week_ending date.

        Returns:
            Array of weights (same length as played_dates), values in range (0, 1]

        Example weights with 3-year (1095 day) half-life:
            - Game from today: 1.00 (100%)
            - Game from 1.5 years ago: ~0.71 (71%)
            - Game from 3 years ago: 0.50 (50%)
            - Game from 6 years ago: 0.25 (25%)
        """
        if reference_date is None:
            reference_date = datetime.now()

        # Convert to numpy array of datetime64 if needed
        if isinstance(played_dates, pd.Series):
            played_dates = played_dates.values

        # Calculate days since each game
        days_ago = np.array([
            (reference_date - pd.Timestamp(date)).days
            for date in played_dates
        ])

        # Ensure no negative days (future games)
        if np.any(days_ago < 0):
            logger.warning(f"Found {np.sum(days_ago < 0)} games with future dates, setting weight to 1.0")
            days_ago = np.maximum(days_ago, 0)

        # Calculate exponential decay weights
        weights = np.exp(-self.lambda_param * days_ago)

        logger.debug(
            f"Calculated {len(weights)} weights. "
            f"Range: {weights.min():.4f} to {weights.max():.4f}, "
            f"Mean: {weights.mean():.4f}"
        )

        return weights

    def get_effective_games_count(self, weights: np.ndarray) -> float:
        """
        Calculate effective number of games after time decay.

        This represents how many "full-weight" games the weighted games
        are equivalent to.

        Args:
            weights: Array of time decay weights

        Returns:
            Effective number of games (sum of weights)

        Example:
            If a player has 200 games but many are old, effective count
            might be 150 (meaning recent games are worth 150 "full" games)
        """
        return float(np.sum(weights))

    def get_weight_for_age(self, days_ago: int) -> float:
        """
        Get the weight for a game played N days ago.

        Utility method for testing or displaying decay curve.

        Args:
            days_ago: Number of days in the past

        Returns:
            Weight value between 0 and 1

        Example:
            >>> decay = TimeDecay(half_life_days=1095)
            >>> decay.get_weight_for_age(1095)  # 3 years ago
            0.5  # Half weight
        """
        if days_ago < 0:
            days_ago = 0
        return np.exp(-self.lambda_param * days_ago)

    def get_decay_info(self) -> dict:
        """
        Get information about the decay parameters.

        Returns:
            Dictionary with decay configuration and example weights
        """
        return {
            "half_life_days": self.half_life_days,
            "half_life_years": self.half_life_days / 365.25,
            "lambda": self.lambda_param,
            "example_weights": {
                "today": self.get_weight_for_age(0),
                "6_months": self.get_weight_for_age(180),
                "1_year": self.get_weight_for_age(365),
                "1.5_years": self.get_weight_for_age(int(1.5 * 365)),
                "3_years": self.get_weight_for_age(1095),
                "6_years": self.get_weight_for_age(2190),
            }
        }
