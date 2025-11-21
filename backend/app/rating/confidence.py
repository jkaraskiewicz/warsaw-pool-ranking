"""New player blending and confidence level calculation."""

from enum import Enum
from typing import Dict
import logging

logger = logging.getLogger(__name__)


class ConfidenceLevel(str, Enum):
    """Player rating confidence levels based on games played."""

    UNRANKED = "unranked"          # <10 games
    PROVISIONAL = "provisional"     # 10-49 games
    EMERGING = "emerging"           # 50-99 games
    ESTABLISHED = "established"     # 100+ games


class PlayerConfidence:
    """
    Handles new player rating blending and confidence level determination.

    New players' ratings are blended between a starter rating and their
    calculated ML rating until they reach the established threshold.
    """

    def __init__(
        self,
        starter_rating: float = 500.0,
        established_games: int = 100,
        min_ranked_games: int = 10
    ):
        """
        Initialize player confidence calculator.

        Args:
            starter_rating: Default rating for new players (default 500)
            established_games: Games needed for fully established rating (default 100)
            min_ranked_games: Minimum games to appear in rankings (default 10)
        """
        self.starter_rating = starter_rating
        self.established_games = established_games
        self.min_ranked_games = min_ranked_games

        logger.info(
            f"PlayerConfidence initialized: starter={starter_rating}, "
            f"established={established_games}, min_ranked={min_ranked_games}"
        )

    def get_confidence_level(self, games_played: int) -> ConfidenceLevel:
        """
        Determine confidence level based on games played.

        Args:
            games_played: Number of games the player has played

        Returns:
            ConfidenceLevel enum value

        Thresholds:
            - Unranked: <10 games
            - Provisional: 10-49 games
            - Emerging: 50-99 games
            - Established: 100+ games
        """
        if games_played < self.min_ranked_games:
            return ConfidenceLevel.UNRANKED
        elif games_played < 50:
            return ConfidenceLevel.PROVISIONAL
        elif games_played < self.established_games:
            return ConfidenceLevel.EMERGING
        else:
            return ConfidenceLevel.ESTABLISHED

    def blend_rating(self, ml_rating: float, games_played: int) -> float:
        """
        Blend ML rating with starter rating for new players.

        For players with fewer than `established_games` games, their displayed
        rating is a weighted blend of the starter rating and their calculated
        ML rating.

        Formula:
            starter_weight = max(0, (established_games - games_played) / established_games)
            ml_weight = 1 - starter_weight
            blended_rating = starter_weight × starter_rating + ml_weight × ml_rating

        Args:
            ml_rating: The Maximum Likelihood calculated rating
            games_played: Number of games the player has played

        Returns:
            Blended rating (float)

        Examples:
            Player with 15 games, ML rating 620:
                starter_weight = (100-15)/100 = 0.85
                ml_weight = 0.15
                blended = 0.85×500 + 0.15×620 = 425 + 93 = 518

            Player with 100+ games:
                Returns ml_rating unchanged (no blending)
        """
        if games_played >= self.established_games:
            # Fully established - return ML rating as-is
            return ml_rating

        # Calculate blend weights
        starter_weight = (self.established_games - games_played) / self.established_games
        ml_weight = 1.0 - starter_weight

        # Blend ratings
        blended = starter_weight * self.starter_rating + ml_weight * ml_rating

        logger.debug(
            f"Blending for {games_played} games: "
            f"ML={ml_rating:.1f}, starter={self.starter_rating:.1f}, "
            f"blended={blended:.1f} (weights: {ml_weight:.2f}/{starter_weight:.2f})"
        )

        return blended

    def get_blend_info(self, games_played: int) -> Dict[str, float]:
        """
        Get information about blend weights for a given number of games.

        Args:
            games_played: Number of games played

        Returns:
            Dictionary with blend information

        Example:
            >>> confidence = PlayerConfidence()
            >>> confidence.get_blend_info(25)
            {
                'games_played': 25,
                'starter_weight': 0.75,
                'ml_weight': 0.25,
                'confidence_level': 'provisional'
            }
        """
        if games_played >= self.established_games:
            starter_weight = 0.0
            ml_weight = 1.0
        else:
            starter_weight = (self.established_games - games_played) / self.established_games
            ml_weight = 1.0 - starter_weight

        return {
            "games_played": games_played,
            "starter_weight": starter_weight,
            "ml_weight": ml_weight,
            "confidence_level": self.get_confidence_level(games_played).value,
            "is_ranked": games_played >= self.min_ranked_games,
        }

    def should_be_ranked(self, games_played: int) -> bool:
        """
        Determine if player should appear in rankings.

        Args:
            games_played: Number of games played

        Returns:
            True if player should be ranked (games >= min_ranked_games)
        """
        return games_played >= self.min_ranked_games
