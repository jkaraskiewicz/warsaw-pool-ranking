import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { PlayerListItem, PlayerDetail, PaginatedResponse } from '../models/player.model';

@Injectable({
  providedIn: 'root'
})
export class PlayerService {
  private apiUrl = '/api';

  constructor(private http: HttpClient) {}

  getPlayers(
    page: number = 1,
    pageSize: number = 100,
    sortBy: string = 'rating',
    order: 'asc' | 'desc' = 'desc',
    filter: string = '',
    ratingType: string = 'all'
  ): Observable<PaginatedResponse<PlayerListItem>> {
    let params = new HttpParams()
      .set('page', page.toString())
      .set('page_size', pageSize.toString())
      .set('sort_by', sortBy)
      .set('order', order)
      .set('rating_type', ratingType);

    if (filter) {
      params = params.set('filter', filter);
    }

    return this.http.get<PaginatedResponse<PlayerListItem>>(`${this.apiUrl}/players`, { params });
  }

  getPlayerDetail(playerId: number, ratingType: string = 'all'): Observable<PlayerDetail> {
    let params = new HttpParams().set('rating_type', ratingType);
    return this.http.get<PlayerDetail>(`${this.apiUrl}/player/${playerId}`, { params });
  }
}
