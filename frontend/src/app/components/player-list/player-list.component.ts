import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
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

import { PlayerListFacade } from './player-list.facade';
import { PlayerListItem } from '../../models/api';
import { PlayerOverlayComponent } from '../player-overlay/player-overlay.component';
import { RatingTypeSelectorComponent } from '../shared/rating-type-selector/rating-type-selector.component';
import { TranslatePipe } from '../../pipes/translate.pipe';

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
    RatingTypeSelectorComponent,
    TranslatePipe
  ],
  templateUrl: './player-list.component.html',
  styleUrls: ['./player-list.component.scss']
})
export class PlayerListComponent {
  displayedColumns: string[] = ['rank', 'name', 'rating', 'games', 'confidence'];
  pageSizeOptions: number[] = [10, 25, 50, 100];
  searchQuery = '';

  constructor(
    public facade: PlayerListFacade,
    private dialog: MatDialog
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
