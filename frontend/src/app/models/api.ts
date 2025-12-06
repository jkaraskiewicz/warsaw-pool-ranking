/* eslint-disable */
export interface PlayerListItem {
  rank: number;
  playerId: number;
  cuescoreId?: number;
  name: string;
  rating: number;
  gamesPlayed: number;
  confidenceLevel: string;
}

export interface PlayerDetail {
  playerId: number;
  cuescoreId?: number;
  name: string;
  cuescoreProfileUrl: string;
  rating: number;
  gamesPlayed: number;
  confidenceLevel: string;
  mlRating: number;
  starterWeight: number;
  mlWeight: number;
  effectiveGames: number;
  lastPlayed?: string;
}

export interface PlayerListResponse {
  items: PlayerListItem[];
  total: number;
  page: number;
  pageSize: number;
}

export interface HeadToHeadMatch {
  date: string;
  tournamentName: string;
  player1Wins: number;
  player2Wins: number;
}

export interface HeadToHeadResponse {
  player1: PlayerDetail | undefined;
  player2: PlayerDetail | undefined;
  probabilityP1Wins: number;
  matches: HeadToHeadMatch[];
}
