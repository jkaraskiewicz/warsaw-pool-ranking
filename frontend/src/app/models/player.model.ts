export interface PlayerListItem {
  rank: number;
  playerId: number;
  cuescoreId: string;
  name: string;
  ratingType: string;
  rating: number;
  gamesPlayed: number;
  confidenceLevel: 'unranked' | 'provisional' | 'emerging' | 'established';
  recentChange: number | null;
}

export interface PlayerDetail {
  playerId: number;
  cuescoreId: string;
  name: string;
  ratingType: string;
  rating: number;
  gamesPlayed: number;
  confidenceLevel: 'unranked' | 'provisional' | 'emerging' | 'established';
  mlRating: number;
  starterWeight: number;
  mlWeight: number;
  effectiveGames: number;
  lastPlayed: string | null;
  recentChange: number | null;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}