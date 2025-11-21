"""Bradley-Terry Maximum Likelihood rating calculator using choix library."""

import numpy as np
import pandas as pd
import choix
from typing import Dict, Optional
import logging

logger = logging.getLogger(__name__)


class RatingCalculator:
    """
    Calculates player ratings using Bradley-Terry Maximum Likelihood model.

    The rating scale is calibrated so that 100 points = 2:1 winning odds,
    matching the FargoRate system.
    """

    def __init__(self, base_rating: float = 500.0):
        """
        Initialize the rating calculator.

        Args:
            base_rating: The center point of the rating scale (default 500)
        """
        self.base_rating = base_rating
        # Scaling factor: 100 points = 2:1 odds
        # ln(2) is approximately 0.693
        self.scale_factor = 100 / np.log(2)

    def calculate_ratings(
        self,
        games_df: pd.DataFrame,
        time_weights: Optional[np.ndarray] = None
    ) -> Dict[int, float]:
        """
        Calculate Maximum Likelihood ratings for all players.

        Args:
            games_df: DataFrame with columns:
                - player_a_id: int
                - player_b_id: int
                - winner_id: int
                - (optional) played_at: datetime for time-weighted calculations
            time_weights: Optional array of weights for each game (same length as games_df)
                          Used for time decay. If None, all games weighted equally.

        Returns:
            Dictionary mapping player_id (int) to rating (float)

        Raises:
            ValueError: If games_df is empty or has invalid data
        """
        if games_df.empty:
            logger.warning("Empty games dataframe provided, returning empty ratings")
            return {}

        # Validate required columns
        required_cols = ['player_a_id', 'player_b_id', 'winner_id']
        missing_cols = set(required_cols) - set(games_df.columns)
        if missing_cols:
            raise ValueError(f"Missing required columns: {missing_cols}")

        # Get all unique players
        all_players = pd.concat([
            games_df['player_a_id'],
            games_df['player_b_id']
        ]).unique()

        n_players = len(all_players)
        logger.info(f"Calculating ratings for {n_players} players from {len(games_df)} games")

        # Create mapping from player_id to index (0-based)
        player_to_idx = {player_id: idx for idx, player_id in enumerate(all_players)}
        idx_to_player = {idx: player_id for player_id, idx in player_to_idx.items()}

        # Prepare pairwise comparisons for choix
        # Format: list of (winner_idx, loser_idx) tuples
        comparisons = []
        weights_list = [] if time_weights is not None else None

        for row_num, (idx, game) in enumerate(games_df.iterrows()):
            winner_idx = player_to_idx[game['winner_id']]

            # Determine loser
            if game['winner_id'] == game['player_a_id']:
                loser_id = game['player_b_id']
            else:
                loser_id = game['player_a_id']

            loser_idx = player_to_idx[loser_id]

            comparisons.append((winner_idx, loser_idx))

            if time_weights is not None:
                weights_list.append(time_weights[row_num])

        # Calculate ML parameters using choix
        # Returns log-strength parameters (one per player)
        try:
            if time_weights is not None:
                # Use LSR (Luce Spectral Ranking) method which supports weights
                params = choix.opt_pairwise(
                    n_players,
                    comparisons,
                    weights=weights_list,
                    method='lsr'  # LSR supports weights
                )
            else:
                # Use default method (faster when no weights)
                params = choix.opt_pairwise(
                    n_players,
                    comparisons,
                    method='lsr'
                )
        except Exception as e:
            logger.error(f"Error in choix optimization: {e}")
            raise

        # Convert log-strength parameters to ratings
        # Center ratings around base_rating (500) and scale appropriately
        ratings = self._params_to_ratings(params, idx_to_player)

        logger.info(f"Ratings calculated. Range: {min(ratings.values()):.1f} to {max(ratings.values()):.1f}")

        return ratings

    def _params_to_ratings(
        self,
        params: np.ndarray,
        idx_to_player: Dict[int, int]
    ) -> Dict[int, float]:
        """
        Convert choix log-strength parameters to human-readable ratings.

        The conversion ensures:
        1. Ratings are centered around base_rating (500)
        2. 100-point difference = 2:1 winning odds

        Args:
            params: Array of log-strength parameters from choix
            idx_to_player: Mapping from array index to player_id

        Returns:
            Dictionary mapping player_id to rating
        """
        # Center parameters around zero
        mean_param = np.mean(params)
        centered_params = params - mean_param

        # Convert to ratings with proper scaling
        # scale_factor ensures 100 pts = 2:1 odds
        ratings = {}
        for idx, param in enumerate(centered_params):
            player_id = idx_to_player[idx]
            rating = self.base_rating + (param * self.scale_factor)
            ratings[player_id] = float(rating)

        return ratings

    def predict_win_probability(self, rating_a: float, rating_b: float) -> float:
        """
        Calculate win probability for player A vs player B.

        Uses the formula:
        P(A wins) = 1 / (1 + 2^((rating_B - rating_A) / 100))

        Args:
            rating_a: Player A's rating
            rating_b: Player B's rating

        Returns:
            Probability that player A wins (0.0 to 1.0)

        Example:
            >>> calc = RatingCalculator()
            >>> calc.predict_win_probability(600, 500)  # 100-pt advantage
            0.667  # Approximately 2:1 odds
        """
        rating_diff = rating_b - rating_a
        return 1.0 / (1.0 + 2.0 ** (rating_diff / 100.0))
