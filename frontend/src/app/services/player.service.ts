import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { PlayerListItem, PlayerDetail, PlayerListResponse, HeadToHeadResponse } from '../models/api';
import { FilterExpression } from '../models/filter';

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
    filters: FilterExpression[] = []
  ): Observable<PlayerListResponse> {
    let params = new HttpParams()
      .set('page', page.toString())
      .set('page_size', pageSize.toString())
      .set('sort_by', sortBy)
      .set('order', order);

    // Build filter DSL string
    if (filters.length > 0) {
      const filterStr = this.buildFilterDsl(filters);
      params = params.set('filter', filterStr);
    }

    return this.http.get<PlayerListResponse>(`${this.apiUrl}/players`, { params });
  }

  private buildFilterDsl(filters: FilterExpression[]): string {
    return filters
      .map(f => {
        if (Array.isArray(f.value)) {
          return `${f.field}:${f.operator}:${f.value.join(',')}`;
        }
        return `${f.field}:${f.operator}:${f.value}`;
      })
      .join('|');
  }

  getPlayerDetail(playerId: number, ratingType: string = 'all'): Observable<PlayerDetail> {
    let params = new HttpParams().set('rating_type', ratingType);
    return this.http.get<PlayerDetail>(`${this.apiUrl}/player/${playerId}`, { params });
  }

  getHeadToHeadComparison(player1Id: number, player2Id: number, ratingType: string = 'all'): Observable<HeadToHeadResponse> {
    let params = new HttpParams().set('rating_type', ratingType);
    return this.http.get<HeadToHeadResponse>(`${this.apiUrl}/compare/${player1Id}/${player2Id}`, { params });
  }
}
