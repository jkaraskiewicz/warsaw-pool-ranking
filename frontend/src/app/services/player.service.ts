import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { PlayerListItem, PlayerDetail, RatingSnapshot } from '../models/player.model';

@Injectable({
  providedIn: 'root'
})
export class PlayerService {
  private apiUrl = '/api';

  constructor(private http: HttpClient) {}

  getPlayers(minGames: number = 10): Observable<PlayerListItem[]> {
    const params = new HttpParams().set('min_games', minGames.toString());
    return this.http.get<PlayerListItem[]>(`${this.apiUrl}/players`, { params });
  }

  getPlayerDetail(playerId: number): Observable<PlayerDetail> {
    return this.http.get<PlayerDetail>(`${this.apiUrl}/player/${playerId}`);
  }

  getPlayerHistory(playerId: number): Observable<RatingSnapshot[]> {
    return this.http.get<RatingSnapshot[]>(`${this.apiUrl}/player/${playerId}/history`);
  }
}
