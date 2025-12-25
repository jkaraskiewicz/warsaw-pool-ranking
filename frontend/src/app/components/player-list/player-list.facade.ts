import { Injectable, signal, computed } from '@angular/core';
import { toObservable } from '@angular/core/rxjs-interop';
import { debounceTime, switchMap, tap } from 'rxjs/operators';
import { PlayerService } from '../../services/player.service';
import { InterestedPlayersService } from '../../services/interested-players.service';
import { PlayerListItem } from '../../models/api';
import { FilterBuilder } from '../../models/filter';

export interface PlayerListState {
  pageIndex: number;
  pageSize: number;
  sortActive: string;
  sortDirection: 'asc' | 'desc';
  nameFilter: string;
  ratingType: string;
  showInterestedOnly: boolean;
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
    nameFilter: '',
    ratingType: 'active',
    showInterestedOnly: false
  });

  // Exposed Signals
  readonly state = this._state.asReadonly();
  readonly loading = signal(true);
  readonly players = signal<PlayerListItem[]>([]);
  readonly total = signal(0);

  // Computed signal for empty state detection
  readonly hasNoResults = computed(() => {
    const players = this.players();
    const loading = this.loading();
    const showInterestedOnly = this._state().showInterestedOnly;

    return players.length === 0 && showInterestedOnly && !loading;
  });

  constructor(
    private playerService: PlayerService,
    private interestedPlayersService: InterestedPlayersService
  ) {
    toObservable(this._state).pipe(
      debounceTime(300),
      tap(() => this.loading.set(true)),
      switchMap(state => {
        // Build filter expressions using FilterBuilder
        const filterBuilder = new FilterBuilder();

        // Rating type filter (always applied)
        filterBuilder.eq('rating_type', state.ratingType);

        // Name search filter (if provided)
        if (state.nameFilter) {
          filterBuilder.contains('name', state.nameFilter);
        }

        // Interested players filter (if enabled)
        if (state.showInterestedOnly) {
          const interestedIds = Array.from(this.interestedPlayersService.interestedPlayerIds());
          if (interestedIds.length > 0) {
            filterBuilder.in('id', interestedIds.map(String));
          }
        }

        return this.playerService.getPlayers(
          state.pageIndex + 1,
          state.pageSize,
          state.sortActive,
          state.sortDirection,
          filterBuilder.build()
        );
      })
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
    this.updateState({ nameFilter: filter, pageIndex: 0 });
  }

  setRatingType(ratingType: string) {
    this.updateState({ ratingType, pageIndex: 0 });
  }

  setShowInterestedOnly(show: boolean) {
    this.updateState({ showInterestedOnly: show, pageIndex: 0 });
  }

  private updateState(partial: Partial<PlayerListState>) {
    this._state.update(current => ({ ...current, ...partial }));
  }
}