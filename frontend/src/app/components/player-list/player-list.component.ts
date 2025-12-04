import { Component, OnInit } from '@angular/core';
import { MatDialog } from '@angular/material/dialog';
import { PageEvent } from '@angular/material/paginator';
import { Sort } from '@angular/material/sort';
import { Subject } from 'rxjs';
import { debounceTime, distinctUntilChanged } from 'rxjs/operators';
import { PlayerService } from '../../services/player.service';
import { PlayerListItem } from '../../models/player.model';
import { PlayerOverlayComponent } from '../player-overlay/player-overlay.component';

@Component({
  selector: 'app-player-list',
  templateUrl: './player-list.component.html',
  styleUrls: ['./player-list.component.scss']
})
export class PlayerListComponent implements OnInit {
  players: PlayerListItem[] = [];
  totalPlayers: number = 0;
  searchQuery: string = '';
  loading: boolean = true;
  displayedColumns: string[] = ['rank', 'name', 'rating', 'games', 'confidence', 'change'];

  // Rating periods
  ratingTypes = [
    { value: 'all', viewValue: 'ALL_TIME' },
    { value: '1y', viewValue: 'LAST_YEAR' },
    { value: '2y', viewValue: 'LAST_2_YEARS' },
    { value: '3y', viewValue: 'LAST_3_YEARS' },
    { value: '4y', viewValue: 'LAST_4_YEARS' },
    { value: '5y', viewValue: 'LAST_5_YEARS' },
  ];
  selectedRatingType: string = 'all';

  // Pagination state
  pageIndex: number = 0;
  pageSize: number = 100;
  pageSizeOptions: number[] = [10, 25, 50, 100];

  // Sorting state
  sortActive: string = 'rating';
  sortDirection: 'asc' | 'desc' = 'desc';

  private searchSubject = new Subject<string>();

  constructor(
    private playerService: PlayerService,
    private dialog: MatDialog
  ) {
    this.searchSubject.pipe(
      debounceTime(400),
      distinctUntilChanged()
    ).subscribe(() => {
      this.pageIndex = 0;
      this.loadPlayers();
    });
  }

  ngOnInit(): void {
    this.loadPlayers();
  }

  loadPlayers(): void {
    this.loading = true;
    // API uses 1-based page, MatPaginator uses 0-based
    this.playerService.getPlayers(
      this.pageIndex + 1,
      this.pageSize,
      this.sortActive,
      this.sortDirection,
      this.searchQuery,
      this.selectedRatingType
    ).subscribe({
      next: (response) => {
        this.players = response.items;
        this.totalPlayers = response.total;
        this.loading = false;
      },
      error: (err) => {
        console.error('Error loading players:', err);
        this.loading = false;
      }
    });
  }

  onSearchChange(): void {
    this.searchSubject.next(this.searchQuery);
  }

  onPageChange(event: PageEvent): void {
    this.pageIndex = event.pageIndex;
    this.pageSize = event.pageSize;
    this.loadPlayers();
  }

  onSortChange(sort: Sort): void {
    this.sortActive = sort.active;
    this.sortDirection = sort.direction === '' ? 'desc' : sort.direction as 'asc' | 'desc';
    this.loadPlayers();
  }

  onRatingTypeChange(type: string): void {
    this.selectedRatingType = type;
    this.pageIndex = 0; // Reset page on rating type change
    this.loadPlayers();
  }

  openPlayerOverlay(player: PlayerListItem): void {
    this.dialog.open(PlayerOverlayComponent, {
      width: '800px',
      maxWidth: '95vw',
      data: { playerId: player.playerId, ratingType: this.selectedRatingType }
    });
  }

  getConfidenceColor(level: string): string {
    switch (level) {
      case 'established':
        return 'primary';
      case 'emerging':
        return 'accent';
      case 'provisional':
        return 'warn';
      default:
        return '';
    }
  }

  getChangeColor(change: number | null): string {
    if (change === null) return '';
    return change > 0 ? 'positive' : change < 0 ? 'negative' : '';
  }

  formatChange(change: number | null): string {
    if (change === null) return '-';
    const sign = change > 0 ? '+' : '';
    return `${sign}${change.toFixed(1)}`;
  }
}
