import { Injectable, signal } from '@angular/core';
import { toObservable } from '@angular/core/rxjs-interop';
import { debounceTime, switchMap, tap } from 'rxjs/operators';
import { PlayerService } from '../../services/player.service';
import { PlayerListItem } from '../../models/api';

export interface PlayerListState {
  pageIndex: number;
  pageSize: number;
  sortActive: string;
  sortDirection: 'asc' | 'desc';
  filter: string;
  ratingType: string;
}

@Injectable({
  providedIn: 'root'
})
export class PlayerListFacade {
  private _state = signal<PlayerListState>({
    pageIndex: 0,
    pageSize: 100,
    sortActive: 'rating',
    sortDirection: 'desc',
    filter: '',
    ratingType: 'all'
  });

  // Exposed Signals
  readonly state = this._state.asReadonly();
  readonly loading = signal(true);
  readonly players = signal<PlayerListItem[]>([]);
  readonly total = signal(0);

  constructor(private playerService: PlayerService) {
    toObservable(this._state).pipe(
      debounceTime(300),
      tap(() => this.loading.set(true)),
      switchMap(state => this.playerService.getPlayers(
        state.pageIndex + 1,
        state.pageSize,
        state.sortActive,
        state.sortDirection,
        state.filter,
        state.ratingType
      ))
    ).subscribe({
      next: (res) => {
        this.players.set(res.items);
        this.total.set(res.total);
        this.loading.set(false);
      },
      error: (err) => {
        console.error('Error loading players', err);
        this.loading.set(false);
      }
    });
  }

  setPage(pageIndex: number, pageSize: number) {
    this.updateState({ pageIndex, pageSize });
  }

  setSort(sortActive: string, sortDirection: 'asc' | 'desc') {
    this.updateState({ sortActive, sortDirection });
  }

  setFilter(filter: string) {
    this.updateState({ filter, pageIndex: 0 });
  }

  setRatingType(ratingType: string) {
    this.updateState({ ratingType, pageIndex: 0 });
  }

  private updateState(partial: Partial<PlayerListState>) {
    this._state.update(current => ({ ...current, ...partial }));
  }
}