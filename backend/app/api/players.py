"""Players API endpoints."""

from fastapi import APIRouter, Depends, HTTPException, Query
from sqlalchemy.orm import Session
from sqlalchemy import func
from typing import List
import logging

from app.database import get_db
from app.models import Player, Rating, RatingSnapshot, Game
from app.schemas import PlayerListItem, PlayerDetail, RatingHistoryPoint, RecentOpponent

logger = logging.getLogger(__name__)

router = APIRouter()


@router.get("/players", response_model=List[PlayerListItem])
def get_players(
    min_games: int = Query(default=10, ge=0, description="Minimum games to be ranked"),
    db: Session = Depends(get_db)
):
    """
    Get list of all ranked players.

    Returns players sorted by rating (highest first).
    Only includes players with games_played >= min_games.

    Args:
        min_games: Minimum games required to appear in list (default 10)
        db: Database session

    Returns:
        List of PlayerListItem objects
    """
    logger.info(f"Fetching players list (min_games={min_games})")

    # Query players with their ratings
    query = (
        db.query(Player, Rating)
        .join(Rating, Player.id == Rating.player_id)
        .filter(Rating.games_played >= min_games)
        .order_by(Rating.rating.desc())
    )

    results = query.all()

    # Calculate recent change (compare to previous week)
    players_list = []
    for rank, (player, rating) in enumerate(results, 1):
        recent_change = _get_recent_change(player.id, db)

        players_list.append(PlayerListItem(
            id=player.id,
            cuescore_id=player.cuescore_id,
            name=player.name,
            rank=rank,
            rating=rating.rating,
            confidence=rating.confidence_level.value,
            games_played=rating.games_played,
            recent_change=recent_change,
            cuescore_url=player.cuescore_profile_url or f"https://cuescore.com/player/{player.name.replace(' ', '+')}/{player.cuescore_id}"
        ))

    logger.info(f"Returning {len(players_list)} ranked players")

    return players_list


@router.get("/player/{player_id}", response_model=PlayerDetail)
def get_player_detail(
    player_id: int,
    db: Session = Depends(get_db)
):
    """
    Get detailed information for a specific player.

    Includes full statistics, best rating, and recent opponents.

    Args:
        player_id: Database player ID
        db: Database session

    Returns:
        PlayerDetail object

    Raises:
        HTTPException 404: If player not found
    """
    logger.info(f"Fetching player detail for ID {player_id}")

    # Get player and rating
    player = db.query(Player).filter(Player.id == player_id).first()
    if not player:
        raise HTTPException(status_code=404, detail="Player not found")

    rating = db.query(Rating).filter(Rating.player_id == player_id).first()
    if not rating:
        raise HTTPException(status_code=404, detail="Player rating not found")

    # Calculate rank
    rank = (
        db.query(func.count(Rating.id))
        .filter(Rating.rating > rating.rating)
        .filter(Rating.games_played >= 10)
        .scalar()
    ) + 1

    # Calculate win percentage
    total_games = rating.total_wins + rating.total_losses
    win_percentage = (rating.total_wins / total_games * 100) if total_games > 0 else 0.0

    # Determine rating trend
    rating_trend = _get_rating_trend(player_id, db)

    # Get recent opponents
    recent_opponents = _get_recent_opponents(player_id, db, limit=5)

    player_detail = PlayerDetail(
        id=player.id,
        cuescore_id=player.cuescore_id,
        name=player.name,
        rating=rating.rating,
        rank=rank,
        confidence=rating.confidence_level.value,
        games_played=rating.games_played,
        total_wins=rating.total_wins,
        total_losses=rating.total_losses,
        win_percentage=round(win_percentage, 1),
        best_rating=rating.best_rating,
        best_rating_date=rating.best_rating_date,
        rating_trend=rating_trend,
        cuescore_url=player.cuescore_profile_url or f"https://cuescore.com/player/{player.name.replace(' ', '+')}/{player.cuescore_id}",
        recent_opponents=recent_opponents
    )

    logger.info(f"Returning player detail for {player.name} (ID {player_id})")

    return player_detail


@router.get("/player/{player_id}/history", response_model=List[RatingHistoryPoint])
def get_player_history(
    player_id: int,
    db: Session = Depends(get_db)
):
    """
    Get rating history for a specific player.

    Returns weekly rating snapshots ordered chronologically.

    Args:
        player_id: Database player ID
        db: Database session

    Returns:
        List of RatingHistoryPoint objects

    Raises:
        HTTPException 404: If player not found
    """
    logger.info(f"Fetching rating history for player ID {player_id}")

    # Verify player exists
    player = db.query(Player).filter(Player.id == player_id).first()
    if not player:
        raise HTTPException(status_code=404, detail="Player not found")

    # Get all snapshots for this player
    snapshots = (
        db.query(RatingSnapshot)
        .filter(RatingSnapshot.player_id == player_id)
        .order_by(RatingSnapshot.week_ending.asc())
        .all()
    )

    history = [
        RatingHistoryPoint(
            week_ending=snapshot.week_ending,
            rating=snapshot.rating,
            games_played=snapshot.games_played
        )
        for snapshot in snapshots
    ]

    logger.info(f"Returning {len(history)} history points for player {player_id}")

    return history


# Helper functions

def _get_recent_change(player_id: int, db: Session) -> float | None:
    """
    Calculate rating change from previous week.

    Args:
        player_id: Player ID
        db: Database session

    Returns:
        Rating change (positive or negative), or None if no history
    """
    # Get last two snapshots
    snapshots = (
        db.query(RatingSnapshot)
        .filter(RatingSnapshot.player_id == player_id)
        .order_by(RatingSnapshot.week_ending.desc())
        .limit(2)
        .all()
    )

    if len(snapshots) < 2:
        return None

    current = snapshots[0].rating
    previous = snapshots[1].rating

    return round(current - previous, 1)


def _get_rating_trend(player_id: int, db: Session) -> str:
    """
    Determine rating trend over last 4 weeks.

    Args:
        player_id: Player ID
        db: Database session

    Returns:
        "improving", "declining", or "stable"
    """
    # Get last 4 weeks of snapshots
    snapshots = (
        db.query(RatingSnapshot)
        .filter(RatingSnapshot.player_id == player_id)
        .order_by(RatingSnapshot.week_ending.desc())
        .limit(4)
        .all()
    )

    if len(snapshots) < 2:
        return "stable"

    # Calculate change from oldest to newest in this window
    oldest = snapshots[-1].rating
    newest = snapshots[0].rating
    change = newest - oldest

    if change > 10:
        return "improving"
    elif change < -10:
        return "declining"
    else:
        return "stable"


def _get_recent_opponents(
    player_id: int,
    db: Session,
    limit: int = 5
) -> List[RecentOpponent]:
    """
    Get list of recent opponents.

    Args:
        player_id: Player ID
        db: Database session
        limit: Maximum number of opponents to return

    Returns:
        List of RecentOpponent objects
    """
    # Get games where player participated
    games = (
        db.query(Game)
        .filter(
            (Game.player_a_id == player_id) | (Game.player_b_id == player_id)
        )
        .order_by(Game.played_at.desc())
        .limit(100)  # Look at last 100 games
        .all()
    )

    # Count games against each opponent
    opponent_counts = {}

    for game in games:
        opponent_id = game.player_b_id if game.player_a_id == player_id else game.player_a_id

        if opponent_id not in opponent_counts:
            opponent_counts[opponent_id] = 0
        opponent_counts[opponent_id] += 1

    # Get top opponents
    top_opponent_ids = sorted(
        opponent_counts.keys(),
        key=lambda x: opponent_counts[x],
        reverse=True
    )[:limit]

    # Fetch opponent details
    recent_opponents = []
    for opponent_id in top_opponent_ids:
        opponent = db.query(Player).filter(Player.id == opponent_id).first()
        if opponent:
            recent_opponents.append(RecentOpponent(
                name=opponent.name,
                games=opponent_counts[opponent_id],
                cuescore_url=opponent.cuescore_profile_url or f"https://cuescore.com/player/{opponent.name.replace(' ', '+')}/{opponent.cuescore_id}"
            ))

    return recent_opponents
