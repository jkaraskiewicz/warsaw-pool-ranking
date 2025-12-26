import { Component, computed, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule, DecimalPipe } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatDialog } from '@angular/material/dialog';
import { PageEvent, MatPaginatorModule } from '@angular/material/paginator';
import { Sort, MatSortModule } from '@angular/material/sort';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatIconModule } from '@angular/material/icon';
import { MatTableModule } from '@angular/material/table';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatChipsModule } from '@angular/material/chips';
import { MatCheckboxModule } from '@angular/material/checkbox';
import { MatButtonModule } from '@angular/material/button';
import { MatBadgeModule } from '@angular/material/badge';
import { MatTooltipModule } from '@angular/material/tooltip';

import { PlayerListFacade } from './player-list.facade';
import { InterestedPlayersService } from '../../services/interested-players.service';
import { PlayerListItem } from '../../models/api';
import { PlayerOverlayComponent } from '../player-overlay/player-overlay.component';
import { ComparisonComponent } from '../comparison/comparison.component';
import { RatingTypeSelectorComponent } from '../shared/rating-type-selector/rating-type-selector.component';
import { TranslatePipe } from '../../pipes/translate.pipe';
import { SkeletonComponent } from '../shared/skeleton/skeleton.component';
import { AvatarComponent } from '../shared/avatar/avatar.component';

@Component({
  selector: 'app-player-list',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatIconModule,
    MatTableModule,
    MatSortModule,
    MatPaginatorModule,
    MatProgressSpinnerModule,
    MatChipsModule,
    MatCheckboxModule,
    MatButtonModule,
    MatBadgeModule,
    MatTooltipModule,
    RatingTypeSelectorComponent,
    TranslatePipe,
    DecimalPipe,
    SkeletonComponent,
    AvatarComponent
  ],
  templateUrl: './player-list.component.html',
  styleUrls: ['./player-list.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerListComponent {
  displayedColumns: string[] = ['select', 'interested', 'rank', 'name', 'rating', 'games', 'matches', 'confidence'];
  pageSizeOptions: number[] = [10, 25, 50, 100];
  searchQuery = '';

  selection = new Set<PlayerListItem>();

  // Computed signals for interested filter UI
  interestedFilterActive = computed(() => this.facade.state().showInterestedOnly);
  interestedCount = computed(() => this.interestedPlayersService.count());

  // Computed signals for stats of current page
  averageRating = computed(() => {
    const players = this.facade.players();
    if (players.length === 0) return 0;
    const totalRating = players.reduce((sum, p) => sum + p.rating, 0);
    return totalRating / players.length;
  });

  highestRating = computed(() => {
    const players = this.facade.players();
    if (players.length === 0) return 0;
    return Math.max(...players.map(p => p.rating));
  });

  totalGames = computed(() => {
    const players = this.facade.players();
    return players.reduce((sum, p) => sum + p.gamesPlayed, 0);
  });

  constructor(
    public facade: PlayerListFacade,
    private dialog: MatDialog,
    public interestedPlayersService: InterestedPlayersService
  ) {}

  onSearchChange(): void {
    this.facade.setFilter(this.searchQuery);
  }

  onPageChange(event: PageEvent): void {
    this.facade.setPage(event.pageIndex, event.pageSize);
  }

  onSortChange(sort: Sort): void {
    this.facade.setSort(sort.active, sort.direction === '' ? 'desc' : sort.direction as 'asc' | 'desc');
  }

  onRatingTypeChange(type: string): void {
    this.facade.setRatingType(type);
  }

  toggleInterestedFilter(): void {
    const current = this.facade.state().showInterestedOnly;
    this.facade.setShowInterestedOnly(!current);
  }

  toggleInterested(event: Event, player: PlayerListItem): void {
    event.stopPropagation();
    this.interestedPlayersService.toggleInterested(player.playerId);
  }

  isInterested(player: PlayerListItem): boolean {
    return this.interestedPlayersService.isInterested(player.playerId);
  }

  clearInterestedPlayers(event: Event): void {
    event.stopPropagation();
    this.interestedPlayersService.clearAll();
  }

  toggleSelection(player: PlayerListItem) {
    if (this.selection.has(player)) {
      this.selection.delete(player);
    } else {
      if (this.selection.size >= 2) {
        // Remove the first added item (FIFOish) to keep selection at max 2
        const first = this.selection.values().next().value;
        this.selection.delete(first);
      }
      this.selection.add(player);
    }
  }

  isSelected(player: PlayerListItem): boolean {
    return this.selection.has(player);
  }

  comparePlayers() {
    if (this.selection.size !== 2) return;
    const players = Array.from(this.selection);
    this.dialog.open(ComparisonComponent, {
      width: '900px',
      maxWidth: '95vw',
      data: {
        player1Id: players[0].playerId,
        player2Id: players[1].playerId,
        ratingType: this.facade.state().ratingType
      }
    });
  }

  openPlayerOverlay(player: PlayerListItem): void {
    this.dialog.open(PlayerOverlayComponent, {
      width: '800px',
      maxWidth: '95vw',
      data: { playerId: player.playerId, ratingType: this.facade.state().ratingType }
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
}
