"""SQLAlchemy models for database tables."""

from datetime import datetime
from enum import Enum as PyEnum

from sqlalchemy import (
    CheckConstraint,
    Column,
    Date,
    DateTime,
    Float,
    ForeignKey,
    Integer,
    String,
    Enum,
)
from sqlalchemy.orm import relationship
from sqlalchemy.sql import func

from app.database import Base


class ConfidenceLevel(str, PyEnum):
    """Player rating confidence levels."""

    UNRANKED = "unranked"
    PROVISIONAL = "provisional"
    EMERGING = "emerging"
    ESTABLISHED = "established"


class Player(Base):
    """Pool player."""

    __tablename__ = "players"

    id = Column(Integer, primary_key=True, index=True)
    cuescore_id = Column(String(50), unique=True, nullable=False, index=True)
    name = Column(String(255), nullable=False, index=True)
    cuescore_profile_url = Column(String(500))
    created_at = Column(DateTime, default=func.now())
    updated_at = Column(DateTime, default=func.now(), onupdate=func.now())

    # Relationships
    rating = relationship("Rating", back_populates="player", uselist=False)
    rating_snapshots = relationship("RatingSnapshot", back_populates="player")
    games_as_player_a = relationship(
        "Game", foreign_keys="Game.player_a_id", back_populates="player_a"
    )
    games_as_player_b = relationship(
        "Game", foreign_keys="Game.player_b_id", back_populates="player_b"
    )
    games_won = relationship(
        "Game", foreign_keys="Game.winner_id", back_populates="winner"
    )


class Venue(Base):
    """Pool venue."""

    __tablename__ = "venues"

    id = Column(Integer, primary_key=True, index=True)
    cuescore_id = Column(String(50), unique=True, nullable=False, index=True)
    name = Column(String(255), nullable=False)
    cuescore_url = Column(String(500))
    created_at = Column(DateTime, default=func.now())
    updated_at = Column(DateTime, default=func.now(), onupdate=func.now())

    # Relationships
    tournaments = relationship("Tournament", back_populates="venue")


class Tournament(Base):
    """Pool tournament."""

    __tablename__ = "tournaments"

    id = Column(Integer, primary_key=True, index=True)
    cuescore_id = Column(String(50), unique=True, nullable=False, index=True)
    name = Column(String(255), nullable=False)
    venue_id = Column(Integer, ForeignKey("venues.id", ondelete="SET NULL"))
    start_date = Column(Date)
    end_date = Column(Date)
    cuescore_url = Column(String(500))
    created_at = Column(DateTime, default=func.now())
    updated_at = Column(DateTime, default=func.now(), onupdate=func.now())

    # Relationships
    venue = relationship("Venue", back_populates="tournaments")
    games = relationship("Game", back_populates="tournament")


class Game(Base):
    """Individual game result."""

    __tablename__ = "games"

    id = Column(Integer, primary_key=True, index=True)
    cuescore_match_id = Column(String(100), nullable=False, index=True)
    tournament_id = Column(
        Integer, ForeignKey("tournaments.id", ondelete="CASCADE"), nullable=False
    )
    player_a_id = Column(
        Integer, ForeignKey("players.id", ondelete="CASCADE"), nullable=False
    )
    player_b_id = Column(
        Integer, ForeignKey("players.id", ondelete="CASCADE"), nullable=False
    )
    winner_id = Column(
        Integer, ForeignKey("players.id", ondelete="CASCADE"), nullable=False
    )
    played_at = Column(DateTime, nullable=False)
    created_at = Column(DateTime, default=func.now())

    # Relationships
    tournament = relationship("Tournament", back_populates="games")
    player_a = relationship(
        "Player", foreign_keys=[player_a_id], back_populates="games_as_player_a"
    )
    player_b = relationship(
        "Player", foreign_keys=[player_b_id], back_populates="games_as_player_b"
    )
    winner = relationship("Player", foreign_keys=[winner_id], back_populates="games_won")

    # Constraint: winner must be one of the players
    __table_args__ = (
        CheckConstraint(
            "(winner_id = player_a_id) OR (winner_id = player_b_id)",
            name="winner_is_player",
        ),
    )


class Rating(Base):
    """Current player rating."""

    __tablename__ = "ratings"

    id = Column(Integer, primary_key=True, index=True)
    player_id = Column(
        Integer,
        ForeignKey("players.id", ondelete="CASCADE"),
        unique=True,
        nullable=False,
        index=True,
    )
    rating = Column(Float, nullable=False)
    games_played = Column(Integer, nullable=False, default=0)
    total_wins = Column(Integer, nullable=False, default=0)
    total_losses = Column(Integer, nullable=False, default=0)
    confidence_level = Column(Enum(ConfidenceLevel), nullable=False)
    best_rating = Column(Float)
    best_rating_date = Column(Date)
    calculated_at = Column(DateTime, nullable=False, default=func.now())

    # Relationships
    player = relationship("Player", back_populates="rating")

    # Constraint: rating must be in reasonable range
    __table_args__ = (
        CheckConstraint("rating >= 0 AND rating <= 2000", name="rating_range"),
    )


class RatingSnapshot(Base):
    """Historical rating snapshot."""

    __tablename__ = "rating_snapshots"

    id = Column(Integer, primary_key=True, index=True)
    player_id = Column(
        Integer, ForeignKey("players.id", ondelete="CASCADE"), nullable=False
    )
    week_ending = Column(Date, nullable=False, index=True)
    rating = Column(Float, nullable=False)
    games_played = Column(Integer, nullable=False)
    confidence_level = Column(Enum(ConfidenceLevel), nullable=False)
    calculation_version = Column(String(10), nullable=False, default="v1")
    created_at = Column(DateTime, default=func.now())

    # Relationships
    player = relationship("Player", back_populates="rating_snapshots")

    # Constraint: rating must be in reasonable range
    __table_args__ = (
        CheckConstraint("rating >= 0 AND rating <= 2000", name="snapshot_rating_range"),
    )
