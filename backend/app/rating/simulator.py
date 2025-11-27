"""Weekly rating simulation engine - replays entire history week by week."""

import pandas as pd
import numpy as np
from datetime import datetime, timedelta
from typing import List, Dict, Tuple
import logging

from app.rating.calculator import RatingCalculator
from app.rating.time_decay import TimeDecay
from app.rating.confidence import PlayerConfidence, ConfidenceLevel

logger = logging.getLogger(__name__)


class WeeklySimulator:
    """
    Simulates rating history by replaying all games week by week.

    Each week, calculates ratings as if running the algorithm on that date,
    using only games played up to that week and applying time decay relative
    to that week's date.

    This approach ensures:
    - Algorithm changes automatically update entire history
    - All snapshots use same algorithm version
    - Rating history is consistent and reproducible
    """

    def __init__(
        self,
        calculator: RatingCalculator = None,
        time_decay: TimeDecay = None,
        confidence: PlayerConfidence = None,
        calculation_version: str = "v1"
    ):
        """
        Initialize the weekly simulator.

        Args:
            calculator: RatingCalculator instance (creates default if None)
            time_decay: TimeDecay instance (creates default if None)
            confidence: PlayerConfidence instance (creates default if None)
            calculation_version: Version tag for this calculation (e.g., "v1", "v2")
        """
        self.calculator = calculator or RatingCalculator()
        self.time_decay = time_decay or TimeDecay()
        self.confidence = confidence or PlayerConfidence()
        self.calculation_version = calculation_version

        logger.info(
            f"WeeklySimulator initialized with calculation version: {calculation_version}"
        )

    def simulate(self, games_df: pd.DataFrame) -> pd.DataFrame:
        """
        Simulate ratings for all historical weeks.

        Args:
            games_df: DataFrame with columns:
                - player_a_id: int
                - player_b_id: int
                - winner_id: int
                - played_at: datetime
                (tournament_id and other columns optional)

        Returns:
            DataFrame with columns:
                - player_id: int
                - week_ending: date (Sunday)
                - rating: float (blended rating)
                - games_played: int
                - confidence_level: str
                - calculation_version: str

        Example:
            >>> games = fetch_all_games()  # DataFrame with ~10,000 games
            >>> simulator = WeeklySimulator()
            >>> snapshots = simulator.simulate(games)
            >>> print(len(snapshots))
            15000  # e.g., 100 players × 150 weeks
        """
        if games_df.empty:
            logger.warning("Empty games dataframe, returning empty snapshots")
            return pd.DataFrame()

        logger.info(f"Starting simulation with {len(games_df)} total games")

        # Ensure played_at is datetime
        games_df['played_at'] = pd.to_datetime(games_df['played_at'])

        # Log date range for debugging
        logger.info(f"Game dates range: {games_df['played_at'].min()} to {games_df['played_at'].max()}")
        logger.info(f"Games with NaT timestamps: {games_df['played_at'].isna().sum()} / {len(games_df)}")

        # Get week boundaries
        week_endings = self._get_week_boundaries(games_df)

        if not week_endings:
            logger.error("No week boundaries generated - all games may have NULL timestamps")
            return pd.DataFrame()

        # For large datasets, sample weeks to reduce computation
        # Calculate snapshots every N weeks instead of every week
        if len(games_df) > 100000:
            snapshot_interval = 12  # Calculate every 12 weeks (~monthly) for very large datasets
            logger.info(
                f"Large dataset detected ({len(games_df)} games) - "
                f"calculating snapshots every {snapshot_interval} weeks (~monthly)"
            )
        else:
            snapshot_interval = 1  # Calculate every week for smaller datasets

        logger.info(
            f"Simulating {len(week_endings)} weeks from {week_endings[0]} to {week_endings[-1]} "
            f"(snapshot interval: every {snapshot_interval} week(s))"
        )

        # Collect all snapshots
        all_snapshots = []

        for idx, week_ending in enumerate(week_endings, 1):
            # Skip weeks based on snapshot interval (but always include last week)
            if idx % snapshot_interval != 0 and idx != len(week_endings):
                continue

            # Get games up to this week
            games_up_to_week = games_df[games_df['played_at'] <= pd.Timestamp(week_ending)]

            if games_up_to_week.empty:
                logger.debug(f"No games for week {week_ending}, skipping")
                continue

            # Log progress with game counts
            logger.info(
                f"Processing snapshot week {idx}/{len(week_endings)} ({idx/len(week_endings)*100:.1f}%): "
                f"{week_ending} - {len(games_up_to_week)} cumulative games"
            )

            # Calculate ratings for this week
            week_snapshots = self._calculate_week_ratings(
                games_up_to_week,
                week_ending
            )

            all_snapshots.extend(week_snapshots)

            # Log milestone progress
            processed_snapshots = (idx // snapshot_interval) + (1 if idx == len(week_endings) and idx % snapshot_interval != 0 else 0)
            if processed_snapshots % 5 == 0:
                logger.info(f"Milestone: Completed {processed_snapshots} snapshot calculations, {len(all_snapshots)} total snapshots generated")

        # Convert to DataFrame
        snapshots_df = pd.DataFrame(all_snapshots)

        logger.info(
            f"Simulation complete: {len(snapshots_df)} snapshots for "
            f"{snapshots_df['player_id'].nunique()} players across "
            f"{snapshots_df['week_ending'].nunique()} weeks"
        )

        return snapshots_df

    def _get_week_boundaries(self, games_df: pd.DataFrame) -> List[datetime]:
        """
        Generate list of week ending dates (Sundays) covering all games.

        Args:
            games_df: DataFrame with 'played_at' column

        Returns:
            List of Sunday dates from first game to most recent Sunday

        Example:
            First game: 2023-01-15 (Monday)
            Last game: 2025-11-18 (Tuesday)
            Returns: [2023-01-15, 2023-01-22, ..., 2025-11-17] (all Sundays)
        """
        first_game_date = games_df['played_at'].min()
        last_game_date = games_df['played_at'].max()

        logger.debug(f"First game date: {first_game_date}, Last game date: {last_game_date}")

        # Check for NaT (Not a Time) values
        if pd.isna(first_game_date) or pd.isna(last_game_date):
            logger.error(f"Invalid date range: first={first_game_date}, last={last_game_date}")
            return []

        # Find the first Sunday on or after first game
        first_sunday = first_game_date
        while first_sunday.weekday() != 6:  # 6 = Sunday
            first_sunday += timedelta(days=1)

        # Find the most recent Sunday (could be today or in past)
        last_sunday = datetime.now()
        while last_sunday.weekday() != 6:
            last_sunday -= timedelta(days=1)

        # Ensure we don't go past the last game if it's in the past
        if last_game_date < pd.Timestamp(last_sunday):
            last_sunday = last_game_date
            while last_sunday.weekday() != 6:
                last_sunday += timedelta(days=1)

        # Generate all Sundays in range
        week_endings = []
        current = first_sunday
        while current <= pd.Timestamp(last_sunday):
            week_endings.append(current)
            current += timedelta(days=7)

        return week_endings

    def _calculate_week_ratings(
        self,
        games_df: pd.DataFrame,
        week_ending: datetime
    ) -> List[Dict]:
        """
        Calculate ratings for all players as of a specific week.

        Args:
            games_df: All games up to and including this week
            week_ending: The Sunday date ending this week (reference date)

        Returns:
            List of snapshot dictionaries for this week

        Process:
            1. Calculate time decay weights relative to week_ending
            2. Run ML rating calculation
            3. Count games per player
            4. Apply new player blending
            5. Determine confidence levels
        """
        # Calculate time decay weights for all games
        time_weights = self.time_decay.calculate_weights(
            games_df['played_at'],
            reference_date=week_ending
        )

        # Calculate ML ratings
        ml_ratings = self.calculator.calculate_ratings(games_df, time_weights)

        # Count games per player (actual games, not weighted)
        games_per_player = self._count_games_per_player(games_df)

        # Generate snapshots with blending and confidence
        snapshots = []
        for player_id, ml_rating in ml_ratings.items():
            games_played = games_per_player.get(player_id, 0)

            # Apply new player blending
            blended_rating = self.confidence.blend_rating(ml_rating, games_played)

            # Determine confidence level
            confidence_level = self.confidence.get_confidence_level(games_played)

            snapshot = {
                'player_id': player_id,
                'week_ending': week_ending.date(),
                'rating': blended_rating,
                'games_played': games_played,
                'confidence_level': confidence_level.value,
                'calculation_version': self.calculation_version
            }

            snapshots.append(snapshot)

        return snapshots

    def _count_games_per_player(self, games_df: pd.DataFrame) -> Dict[int, int]:
        """
        Count total games played by each player.

        Args:
            games_df: DataFrame with player_a_id and player_b_id columns

        Returns:
            Dictionary mapping player_id to games count

        Note:
            Each game counts once per player (not per win/loss)
        """
        player_game_counts = {}

        # Count games as player_a
        for player_id in games_df['player_a_id']:
            player_game_counts[player_id] = player_game_counts.get(player_id, 0) + 1

        # Count games as player_b
        for player_id in games_df['player_b_id']:
            player_game_counts[player_id] = player_game_counts.get(player_id, 0) + 1

        return player_game_counts

    def calculate_current_ratings(
        self,
        games_df: pd.DataFrame
    ) -> Tuple[pd.DataFrame, pd.DataFrame]:
        """
        Calculate current ratings (as of now) with full statistics.

        This is used to update the 'ratings' table with current values.

        Args:
            games_df: All games in database

        Returns:
            Tuple of (ratings_df, snapshots_df):
                - ratings_df: Current ratings with full stats
                - snapshots_df: All historical weekly snapshots

        The ratings_df includes:
            - player_id
            - rating (blended)
            - games_played
            - total_wins
            - total_losses
            - confidence_level
            - best_rating (from historical snapshots)
            - best_rating_date
            - calculated_at (timestamp)
        """
        logger.info("Calculating current ratings with full statistics")

        # Run full simulation to get snapshots
        snapshots_df = self.simulate(games_df)

        if snapshots_df.empty:
            return pd.DataFrame(), pd.DataFrame()

        # Get current (latest week) ratings
        latest_week = snapshots_df['week_ending'].max()
        current_ratings = snapshots_df[snapshots_df['week_ending'] == latest_week].copy()

        # Calculate win/loss counts
        win_loss_stats = self._calculate_win_loss_stats(games_df)
        current_ratings = current_ratings.merge(
            win_loss_stats,
            on='player_id',
            how='left'
        )

        # Calculate best rating from history
        best_ratings = snapshots_df.groupby('player_id').agg({
            'rating': 'max'
        }).reset_index()
        best_ratings.columns = ['player_id', 'best_rating']

        # Get date of best rating
        best_rating_dates = []
        for _, row in best_ratings.iterrows():
            player_id = row['player_id']
            best_rating = row['best_rating']

            # Find first occurrence of best rating
            player_snaps = snapshots_df[snapshots_df['player_id'] == player_id]
            best_snap = player_snaps[player_snaps['rating'] == best_rating].iloc[0]
            best_rating_dates.append({
                'player_id': player_id,
                'best_rating_date': best_snap['week_ending']
            })

        best_dates_df = pd.DataFrame(best_rating_dates)
        current_ratings = current_ratings.merge(best_dates_df, on='player_id', how='left')
        current_ratings = current_ratings.merge(best_ratings, on='player_id', how='left')

        # Add calculated_at timestamp
        current_ratings['calculated_at'] = datetime.now()

        # Select final columns
        current_ratings = current_ratings[[
            'player_id', 'rating', 'games_played',
            'total_wins', 'total_losses', 'confidence_level',
            'best_rating', 'best_rating_date', 'calculated_at'
        ]]

        logger.info(f"Current ratings calculated for {len(current_ratings)} players")

        return current_ratings, snapshots_df

    def _calculate_win_loss_stats(self, games_df: pd.DataFrame) -> pd.DataFrame:
        """
        Calculate total wins and losses for each player.

        Args:
            games_df: All games

        Returns:
            DataFrame with player_id, total_wins, total_losses
        """
        stats = []

        # Get all unique players
        all_players = pd.concat([
            games_df['player_a_id'],
            games_df['player_b_id']
        ]).unique()

        for player_id in all_players:
            # Count wins
            wins = len(games_df[games_df['winner_id'] == player_id])

            # Count losses (games where player participated but didn't win)
            played_as_a = games_df[games_df['player_a_id'] == player_id]
            played_as_b = games_df[games_df['player_b_id'] == player_id]

            losses_as_a = len(played_as_a[played_as_a['winner_id'] != player_id])
            losses_as_b = len(played_as_b[played_as_b['winner_id'] != player_id])
            losses = losses_as_a + losses_as_b

            stats.append({
                'player_id': player_id,
                'total_wins': wins,
                'total_losses': losses
            })

        return pd.DataFrame(stats)
