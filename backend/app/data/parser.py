"""Parser for converting CueScore tournament data to game records."""

from typing import List, Dict, Optional
from datetime import datetime
import logging

logger = logging.getLogger(__name__)


class TournamentParser:
    """
    Parses CueScore tournament API responses into database-ready records.

    Converts match-level data (e.g., "7-5") into individual game records
    (e.g., 7 games won by player A, 5 games won by player B).

    Filters out non-pool disciplines (snooker, pyramid).
    """

    # Disciplines to exclude (snooker, pyramid variants)
    EXCLUDED_DISCIPLINES = [
        'snooker',
        'pyramid',
        'piramida',  # Polish
        'russian pyramid',
        'russian pool',
    ]

    def parse_tournament(self, tournament_data: Dict) -> Optional[Dict]:
        """
        Parse tournament data into database-ready format.

        Filters out non-pool tournaments (snooker, pyramid).

        Args:
            tournament_data: Raw tournament data from CueScore API

        Returns:
            Dictionary with:
                - tournament_info: Tournament metadata
                - participants: List of player records
                - games: List of game records
            Or None if tournament should be excluded (non-pool discipline)

        Raises:
            ValueError: If tournament data is invalid or missing required fields
        """
        if not tournament_data:
            raise ValueError("Empty tournament data")

        tournament_id = tournament_data.get('id')
        if not tournament_id:
            raise ValueError("Tournament missing 'id' field")

        # Check discipline - skip non-pool tournaments
        discipline = tournament_data.get('discipline', '').lower()
        if self._is_excluded_discipline(discipline):
            logger.info(
                f"Skipping tournament {tournament_id} ({tournament_data.get('name', 'Unknown')}): "
                f"discipline '{discipline}' is not pool"
            )
            return None

        logger.info(f"Parsing tournament {tournament_id}: {tournament_data.get('name', 'Unknown')}")

        # Parse tournament metadata
        tournament_info = self._parse_tournament_info(tournament_data)

        # Parse participants
        participants = self._parse_participants(tournament_data)

        # Parse matches into games
        games = self._parse_matches_to_games(tournament_data, tournament_id)

        logger.info(
            f"Parsed tournament {tournament_id}: "
            f"{len(participants)} participants, {len(games)} games"
        )

        return {
            'tournament_info': tournament_info,
            'participants': participants,
            'games': games
        }

    def _parse_tournament_info(self, data: Dict) -> Dict:
        """
        Extract tournament metadata.

        Args:
            data: Raw tournament data

        Returns:
            Tournament info dictionary with keys:
                - cuescore_id, name, start_date, end_date
        """
        return {
            'cuescore_id': data.get('id'),
            'name': data.get('name'),
            'start_date': self._parse_date(data.get('startDate')),
            'end_date': self._parse_date(data.get('endDate')),
        }

    def _parse_participants(self, data: Dict) -> List[Dict]:
        """
        Extract participant/player information from matches.

        Since CueScore API doesn't have a separate 'participants' field,
        we extract unique players from the matches themselves.

        Args:
            data: Raw tournament data with 'matches' field

        Returns:
            List of player dictionaries with keys:
                - cuescore_id, name, cuescore_profile_url
        """
        matches = data.get('matches', [])
        if not matches:
            logger.warning(f"Tournament {data.get('id')} has no matches")
            return []

        # Extract unique players from matches
        players_dict = {}  # Use dict to deduplicate by player_id

        for match in matches:
            # Extract playerA
            player_a = match.get('playerA', {})
            if isinstance(player_a, dict):
                player_a_id = player_a.get('playerId')
                player_a_name = player_a.get('name')
                if player_a_id and player_a_name:
                    players_dict[str(player_a_id)] = {
                        'cuescore_id': str(player_a_id),
                        'name': player_a_name,
                        'cuescore_profile_url': f"https://cuescore.com/player/{player_a_name.replace(' ', '+')}/{player_a_id}"
                    }

            # Extract playerB
            player_b = match.get('playerB', {})
            if isinstance(player_b, dict):
                player_b_id = player_b.get('playerId')
                player_b_name = player_b.get('name')
                if player_b_id and player_b_name:
                    players_dict[str(player_b_id)] = {
                        'cuescore_id': str(player_b_id),
                        'name': player_b_name,
                        'cuescore_profile_url': f"https://cuescore.com/player/{player_b_name.replace(' ', '+')}/{player_b_id}"
                    }

        return list(players_dict.values())

    def _parse_matches_to_games(
        self,
        data: Dict,
        tournament_id: str
    ) -> List[Dict]:
        """
        Parse match results into individual game records.

        Converts match-level scores (e.g., "7-5") into game-level records.

        Args:
            data: Raw tournament data with 'matches' field
            tournament_id: Tournament ID for reference

        Returns:
            List of game dictionaries with keys:
                - cuescore_match_id, tournament_cuescore_id,
                  player_a_cuescore_id, player_b_cuescore_id,
                  winner_cuescore_id, played_at
        """
        matches = data.get('matches', [])
        if not matches:
            logger.warning(f"Tournament {tournament_id} has no matches")
            return []

        all_games = []

        for match in matches:
            try:
                match_games = self._parse_single_match(match, tournament_id)
                all_games.extend(match_games)
            except Exception as e:
                logger.error(f"Failed to parse match in tournament {tournament_id}: {e}")
                continue

        return all_games

    def _parse_single_match(
        self,
        match: Dict,
        tournament_id: str
    ) -> List[Dict]:
        """
        Parse a single match into individual games.

        Args:
            match: Match data from API
            tournament_id: Tournament ID

        Returns:
            List of game records

        Example:
            Match with score 7-5 (Player A vs Player B, A wins) creates:
            - 7 game records with winner = Player A
            - 5 game records with winner = Player B
        """
        # Extract match data
        match_id = match.get('id') or match.get('matchId')

        # Player IDs can be nested in playerA/playerB objects
        player_a_data = match.get('playerA', {})
        player_b_data = match.get('playerB', {})
        player_a_id = (
            match.get('player1Id') or
            match.get('playerAId') or
            (player_a_data.get('playerId') if isinstance(player_a_data, dict) else None)
        )
        player_b_id = (
            match.get('player2Id') or
            match.get('playerBId') or
            (player_b_data.get('playerId') if isinstance(player_b_data, dict) else None)
        )

        # Score can be in different formats
        score_a = match.get('score1') or match.get('scoreA')
        score_b = match.get('score2') or match.get('scoreB')

        # Played date/time
        played_at = self._parse_datetime(
            match.get('starttime') or match.get('playedAt') or match.get('date') or match.get('timestamp')
        )

        # Validate required fields
        if not all([match_id, player_a_id, player_b_id]):
            raise ValueError(f"Match missing required fields: {match}")

        if score_a is None or score_b is None:
            logger.warning(f"Match {match_id} has no scores, skipping")
            return []

        # Convert scores to integers
        try:
            score_a = int(score_a)
            score_b = int(score_b)
        except (ValueError, TypeError):
            logger.warning(f"Match {match_id} has invalid scores: {score_a}, {score_b}")
            return []

        # Generate individual game records
        games = []

        # Create score_a games won by player A
        for _ in range(score_a):
            games.append({
                'cuescore_match_id': str(match_id),
                'tournament_cuescore_id': str(tournament_id),
                'player_a_cuescore_id': str(player_a_id),
                'player_b_cuescore_id': str(player_b_id),
                'winner_cuescore_id': str(player_a_id),
                'played_at': played_at
            })

        # Create score_b games won by player B
        for _ in range(score_b):
            games.append({
                'cuescore_match_id': str(match_id),
                'tournament_cuescore_id': str(tournament_id),
                'player_a_cuescore_id': str(player_a_id),
                'player_b_cuescore_id': str(player_b_id),
                'winner_cuescore_id': str(player_b_id),
                'played_at': played_at
            })

        logger.debug(
            f"Match {match_id}: {score_a}-{score_b} → {len(games)} game records"
        )

        return games

    def _parse_date(self, date_str: Optional[str]) -> Optional[datetime.date]:
        """
        Parse date string to date object.

        Args:
            date_str: Date string (various formats supported)

        Returns:
            datetime.date or None
        """
        if not date_str:
            return None

        try:
            # Try common formats
            for fmt in ['%Y-%m-%d', '%d-%m-%Y', '%m/%d/%Y']:
                try:
                    return datetime.strptime(date_str, fmt).date()
                except ValueError:
                    continue

            logger.warning(f"Could not parse date: {date_str}")
            return None

        except Exception as e:
            logger.error(f"Error parsing date {date_str}: {e}")
            return None

    def _parse_datetime(self, dt_str: Optional[str]) -> Optional[datetime]:
        """
        Parse datetime string to datetime object.

        Args:
            dt_str: Datetime string or timestamp

        Returns:
            datetime object or None
        """
        if not dt_str:
            # If no timestamp, use a default (will be updated based on tournament date)
            return None

        try:
            # Try ISO format
            if 'T' in str(dt_str):
                return datetime.fromisoformat(dt_str.replace('Z', '+00:00'))

            # Try timestamp (seconds)
            if isinstance(dt_str, (int, float)):
                return datetime.fromtimestamp(dt_str)

            # Try string timestamp
            try:
                timestamp = float(dt_str)
                return datetime.fromtimestamp(timestamp)
            except ValueError:
                pass

            logger.warning(f"Could not parse datetime: {dt_str}")
            return None

        except Exception as e:
            logger.error(f"Error parsing datetime {dt_str}: {e}")
            return None

    def _is_excluded_discipline(self, discipline: str) -> bool:
        """
        Check if tournament discipline should be excluded.

        Args:
            discipline: Tournament discipline (e.g., 'pool', 'snooker', 'pyramid')

        Returns:
            True if discipline should be excluded (not pool)
        """
        if not discipline:
            # If no discipline specified, assume it's pool
            return False

        discipline_lower = discipline.lower().strip()

        # Check against excluded disciplines
        for excluded in self.EXCLUDED_DISCIPLINES:
            if excluded in discipline_lower:
                return True

        return False
