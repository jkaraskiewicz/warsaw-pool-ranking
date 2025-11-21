"""Pydantic schemas for API request/response validation."""

from pydantic import BaseModel, Field
from datetime import datetime, date
from typing import List, Optional


class PlayerListItem(BaseModel):
    """Player item in the main list."""

    id: int
    cuescore_id: str
    name: str
    rank: int
    rating: float
    confidence: str
    games_played: int
    recent_change: Optional[float] = None
    cuescore_url: str

    class Config:
        from_attributes = True


class RecentOpponent(BaseModel):
    """Recent opponent information."""

    name: str
    games: int
    cuescore_url: str


class PlayerDetail(BaseModel):
    """Detailed player information for overlay/dialog."""

    id: int
    cuescore_id: str
    name: str
    rating: float
    rank: int
    confidence: str
    games_played: int
    total_wins: int
    total_losses: int
    win_percentage: float
    best_rating: Optional[float] = None
    best_rating_date: Optional[date] = None
    rating_trend: str  # "improving", "declining", "stable"
    cuescore_url: str
    recent_opponents: List[RecentOpponent] = []

    class Config:
        from_attributes = True


class RatingHistoryPoint(BaseModel):
    """Single point in rating history chart."""

    week_ending: date
    rating: float
    games_played: int

    class Config:
        from_attributes = True


class HealthCheck(BaseModel):
    """Health check response."""

    status: str
    timestamp: datetime = Field(default_factory=datetime.now)


class APIInfo(BaseModel):
    """API information response."""

    message: str
    version: str
    status: str
