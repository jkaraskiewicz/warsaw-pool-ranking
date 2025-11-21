"""Tests for tournament parser including discipline filtering."""

import pytest
from app.data.parser import TournamentParser


class TestTournamentParser:
    """Test suite for TournamentParser."""

    def test_parse_pool_tournament(self):
        """Test parsing a valid pool tournament."""
        parser = TournamentParser()

        tournament_data = {
            'id': '123',
            'name': '8-Ball Championship',
            'discipline': 'pool',
            'startDate': '2025-01-01',
            'endDate': '2025-01-02',
            'participants': [
                {'id': '1', 'name': 'Player A'},
                {'id': '2', 'name': 'Player B'}
            ],
            'matches': [
                {
                    'id': 'm1',
                    'player1Id': '1',
                    'player2Id': '2',
                    'score1': 7,
                    'score2': 5,
                    'playedAt': '2025-01-01T12:00:00'
                }
            ]
        }

        result = parser.parse_tournament(tournament_data)

        assert result is not None
        assert result['tournament_info']['cuescore_id'] == '123'
        assert len(result['participants']) == 2
        assert len(result['games']) == 12  # 7 + 5 games

    def test_filter_snooker_tournament(self):
        """Test that snooker tournaments are filtered out."""
        parser = TournamentParser()

        tournament_data = {
            'id': '124',
            'name': 'Snooker Championship',
            'discipline': 'snooker',
            'startDate': '2025-01-01',
            'participants': [],
            'matches': []
        }

        result = parser.parse_tournament(tournament_data)

        assert result is None

    def test_filter_pyramid_tournament(self):
        """Test that pyramid tournaments are filtered out."""
        parser = TournamentParser()

        tournament_data = {
            'id': '125',
            'name': 'Pyramid Championship',
            'discipline': 'pyramid',
            'startDate': '2025-01-01',
            'participants': [],
            'matches': []
        }

        result = parser.parse_tournament(tournament_data)

        assert result is None

    def test_filter_piramida_polish(self):
        """Test that Polish 'piramida' tournaments are filtered out."""
        parser = TournamentParser()

        tournament_data = {
            'id': '126',
            'name': 'Turniej Piramida',
            'discipline': 'piramida',
            'startDate': '2025-01-01',
            'participants': [],
            'matches': []
        }

        result = parser.parse_tournament(tournament_data)

        assert result is None

    def test_filter_russian_pyramid(self):
        """Test that Russian pyramid tournaments are filtered out."""
        parser = TournamentParser()

        tournament_data = {
            'id': '127',
            'name': 'Russian Pyramid Championship',
            'discipline': 'Russian Pyramid',
            'startDate': '2025-01-01',
            'participants': [],
            'matches': []
        }

        result = parser.parse_tournament(tournament_data)

        assert result is None

    def test_no_discipline_assumes_pool(self):
        """Test that tournaments without discipline are assumed to be pool."""
        parser = TournamentParser()

        tournament_data = {
            'id': '128',
            'name': 'Tournament',
            # No discipline field
            'startDate': '2025-01-01',
            'participants': [
                {'id': '1', 'name': 'Player A'},
            ],
            'matches': []
        }

        result = parser.parse_tournament(tournament_data)

        # Should not be filtered out (assumes pool)
        assert result is not None
        assert result['tournament_info']['cuescore_id'] == '128'

    def test_empty_discipline_assumes_pool(self):
        """Test that tournaments with empty discipline are assumed to be pool."""
        parser = TournamentParser()

        tournament_data = {
            'id': '129',
            'name': 'Tournament',
            'discipline': '',
            'startDate': '2025-01-01',
            'participants': [
                {'id': '1', 'name': 'Player A'},
            ],
            'matches': []
        }

        result = parser.parse_tournament(tournament_data)

        # Should not be filtered out (assumes pool)
        assert result is not None

    def test_case_insensitive_filtering(self):
        """Test that discipline filtering is case-insensitive."""
        parser = TournamentParser()

        # Test various case variations
        for discipline in ['SNOOKER', 'Snooker', 'sNoOkEr', 'PYRAMID', 'Pyramid']:
            tournament_data = {
                'id': '130',
                'name': 'Tournament',
                'discipline': discipline,
                'participants': [],
                'matches': []
            }

            result = parser.parse_tournament(tournament_data)
            assert result is None, f"Failed to filter discipline: {discipline}"

    def test_discipline_substring_matching(self):
        """Test that discipline filtering works with substrings."""
        parser = TournamentParser()

        # Should filter out if "snooker" appears anywhere
        tournament_data = {
            'id': '131',
            'name': 'Tournament',
            'discipline': 'Pool & Snooker',
            'participants': [],
            'matches': []
        }

        result = parser.parse_tournament(tournament_data)
        assert result is None
