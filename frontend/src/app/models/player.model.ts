export interface PlayerListItem {
  rank: number;
  player_id: number;
  cuescore_id: string;
  name: string;
  rating: number;
  games_played: number;
  confidence_level: 'unranked' | 'provisional' | 'emerging' | 'established';
  recent_change: number | null;
}

export interface PlayerDetail {
  player_id: number;
  cuescore_id: string;
  name: string;
  rating: number;
  games_played: number;
  confidence_level: 'unranked' | 'provisional' | 'emerging' | 'established';
  ml_rating: number;
  starter_weight: number;
  ml_weight: number;
  effective_games: number;
  last_played: string | null;
  recent_change: number | null;
}

export interface RatingSnapshot {
  week_ending: string;
  rating: number;
  games_played: number;
  confidence_level: 'unranked' | 'provisional' | 'emerging' | 'established';
}
