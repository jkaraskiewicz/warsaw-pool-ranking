"""Tests for player confidence and rating blending."""

import pytest
from app.rating.confidence import PlayerConfidence, ConfidenceLevel


class TestPlayerConfidence:
    """Test suite for PlayerConfidence."""

    def test_initialization(self):
        """Test confidence calculator initialization."""
        confidence = PlayerConfidence(
            starter_rating=600,
            established_games=150,
            min_ranked_games=15
        )

        assert confidence.starter_rating == 600
        assert confidence.established_games == 150
        assert confidence.min_ranked_games == 15

    def test_default_initialization(self):
        """Test with default parameters."""
        confidence = PlayerConfidence()

        assert confidence.starter_rating == 500.0
        assert confidence.established_games == 100
        assert confidence.min_ranked_games == 10

    def test_confidence_level_unranked(self):
        """Test confidence level for unranked players."""
        confidence = PlayerConfidence()

        # Less than 10 games → Unranked
        assert confidence.get_confidence_level(0) == ConfidenceLevel.UNRANKED
        assert confidence.get_confidence_level(5) == ConfidenceLevel.UNRANKED
        assert confidence.get_confidence_level(9) == ConfidenceLevel.UNRANKED

    def test_confidence_level_provisional(self):
        """Test confidence level for provisional players."""
        confidence = PlayerConfidence()

        # 10-49 games → Provisional
        assert confidence.get_confidence_level(10) == ConfidenceLevel.PROVISIONAL
        assert confidence.get_confidence_level(25) == ConfidenceLevel.PROVISIONAL
        assert confidence.get_confidence_level(49) == ConfidenceLevel.PROVISIONAL

    def test_confidence_level_emerging(self):
        """Test confidence level for emerging players."""
        confidence = PlayerConfidence()

        # 50-99 games → Emerging
        assert confidence.get_confidence_level(50) == ConfidenceLevel.EMERGING
        assert confidence.get_confidence_level(75) == ConfidenceLevel.EMERGING
        assert confidence.get_confidence_level(99) == ConfidenceLevel.EMERGING

    def test_confidence_level_established(self):
        """Test confidence level for established players."""
        confidence = PlayerConfidence()

        # 100+ games → Established
        assert confidence.get_confidence_level(100) == ConfidenceLevel.ESTABLISHED
        assert confidence.get_confidence_level(150) == ConfidenceLevel.ESTABLISHED
        assert confidence.get_confidence_level(1000) == ConfidenceLevel.ESTABLISHED

    def test_blend_rating_no_games(self):
        """Test blending with zero games."""
        confidence = PlayerConfidence(starter_rating=500)

        # 0 games → 100% starter rating
        blended = confidence.blend_rating(ml_rating=700, games_played=0)
        assert blended == 500.0

    def test_blend_rating_few_games(self):
        """Test blending with few games."""
        confidence = PlayerConfidence(starter_rating=500, established_games=100)

        # 15 games → mostly starter, some ML
        ml_rating = 620
        blended = confidence.blend_rating(ml_rating=ml_rating, games_played=15)

        # Should be between starter and ML, closer to starter
        assert 500 < blended < 620
        assert abs(blended - 500) < abs(blended - 620)

        # Calculate expected value
        starter_weight = (100 - 15) / 100  # 0.85
        ml_weight = 0.15
        expected = starter_weight * 500 + ml_weight * 620  # 518
        assert abs(blended - expected) < 0.01

    def test_blend_rating_half_established(self):
        """Test blending at 50% of established threshold."""
        confidence = PlayerConfidence(starter_rating=500, established_games=100)

        # 50 games → 50/50 blend
        blended = confidence.blend_rating(ml_rating=600, games_played=50)

        expected = 0.5 * 500 + 0.5 * 600  # 550
        assert abs(blended - expected) < 0.01

    def test_blend_rating_nearly_established(self):
        """Test blending close to established threshold."""
        confidence = PlayerConfidence(starter_rating=500, established_games=100)

        # 95 games → mostly ML
        blended = confidence.blend_rating(ml_rating=700, games_played=95)

        # Should be close to ML rating
        assert 650 < blended < 700
        assert abs(blended - 700) < abs(blended - 500)

    def test_blend_rating_established(self):
        """Test blending for established players."""
        confidence = PlayerConfidence(starter_rating=500, established_games=100)

        # 100+ games → pure ML rating (no blending)
        assert confidence.blend_rating(ml_rating=700, games_played=100) == 700
        assert confidence.blend_rating(ml_rating=650, games_played=150) == 650
        assert confidence.blend_rating(ml_rating=800, games_played=1000) == 800

    def test_blend_rating_various_ml_ratings(self):
        """Test blending with various ML ratings."""
        confidence = PlayerConfidence(starter_rating=500, established_games=100)

        games_played = 25  # 75% starter, 25% ML

        # High ML rating
        blended_high = confidence.blend_rating(ml_rating=800, games_played=games_played)
        expected_high = 0.75 * 500 + 0.25 * 800  # 575
        assert abs(blended_high - expected_high) < 0.01

        # Low ML rating
        blended_low = confidence.blend_rating(ml_rating=300, games_played=games_played)
        expected_low = 0.75 * 500 + 0.25 * 300  # 450
        assert abs(blended_low - expected_low) < 0.01

    def test_get_blend_info_zero_games(self):
        """Test blend info for new player."""
        confidence = PlayerConfidence()

        info = confidence.get_blend_info(0)

        assert info['games_played'] == 0
        assert info['starter_weight'] == 1.0
        assert info['ml_weight'] == 0.0
        assert info['confidence_level'] == 'unranked'
        assert info['is_ranked'] == False

    def test_get_blend_info_provisional(self):
        """Test blend info for provisional player."""
        confidence = PlayerConfidence()

        info = confidence.get_blend_info(25)

        assert info['games_played'] == 25
        assert info['starter_weight'] == 0.75
        assert info['ml_weight'] == 0.25
        assert info['confidence_level'] == 'provisional'
        assert info['is_ranked'] == True

    def test_get_blend_info_established(self):
        """Test blend info for established player."""
        confidence = PlayerConfidence()

        info = confidence.get_blend_info(100)

        assert info['games_played'] == 100
        assert info['starter_weight'] == 0.0
        assert info['ml_weight'] == 1.0
        assert info['confidence_level'] == 'established'
        assert info['is_ranked'] == True

    def test_should_be_ranked(self):
        """Test ranking eligibility."""
        confidence = PlayerConfidence(min_ranked_games=10)

        # Below threshold → not ranked
        assert confidence.should_be_ranked(0) == False
        assert confidence.should_be_ranked(5) == False
        assert confidence.should_be_ranked(9) == False

        # At or above threshold → ranked
        assert confidence.should_be_ranked(10) == True
        assert confidence.should_be_ranked(50) == True
        assert confidence.should_be_ranked(100) == True

    def test_custom_thresholds(self):
        """Test with custom threshold values."""
        confidence = PlayerConfidence(
            starter_rating=600,
            established_games=200,
            min_ranked_games=20
        )

        # Test custom min_ranked_games
        assert confidence.should_be_ranked(15) == False
        assert confidence.should_be_ranked(20) == True

        # Test custom established_games for confidence levels
        assert confidence.get_confidence_level(15) == ConfidenceLevel.UNRANKED
        assert confidence.get_confidence_level(30) == ConfidenceLevel.PROVISIONAL
        assert confidence.get_confidence_level(75) == ConfidenceLevel.EMERGING
        assert confidence.get_confidence_level(200) == ConfidenceLevel.ESTABLISHED

        # Test custom starter_rating in blending
        blended = confidence.blend_rating(ml_rating=700, games_played=100)
        # 50% blend: 0.5 * 600 + 0.5 * 700 = 650
        assert abs(blended - 650) < 0.01

    def test_blend_rating_monotonic(self):
        """Test that blended rating changes monotonically with games played."""
        confidence = PlayerConfidence(starter_rating=500)

        ml_rating = 700  # Higher than starter

        blended_values = []
        for games in [0, 25, 50, 75, 100]:
            blended = confidence.blend_rating(ml_rating=ml_rating, games_played=games)
            blended_values.append(blended)

        # Blended rating should increase monotonically
        # (since ML rating > starter rating)
        for i in range(len(blended_values) - 1):
            assert blended_values[i] < blended_values[i+1]

    def test_confidence_level_enum_values(self):
        """Test that confidence level enum has correct string values."""
        assert ConfidenceLevel.UNRANKED.value == "unranked"
        assert ConfidenceLevel.PROVISIONAL.value == "provisional"
        assert ConfidenceLevel.EMERGING.value == "emerging"
        assert ConfidenceLevel.ESTABLISHED.value == "established"

    def test_blend_rating_example_from_design(self):
        """Test the example from DESIGN.md."""
        confidence = PlayerConfidence(starter_rating=500, established_games=100)

        # Example: 15 games, ML rating 620
        blended = confidence.blend_rating(ml_rating=620, games_played=15)

        # Expected: 0.85×500 + 0.15×620 = 425 + 93 = 518
        assert abs(blended - 518) < 0.1
