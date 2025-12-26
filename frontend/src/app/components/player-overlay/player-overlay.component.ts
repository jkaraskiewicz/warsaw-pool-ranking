import { Component, OnInit, Inject, ChangeDetectionStrategy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MAT_DIALOG_DATA, MatDialogRef, MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatChipsModule } from '@angular/material/chips';
import { PlayerService } from '../../services/player.service';
import { PlayerDetail, PlayerRivalriesResponse } from '../../models/api';
import { TranslatePipe } from '../../pipes/translate.pipe';
import { SkeletonComponent } from '../shared/skeleton/skeleton.component';
import { AvatarComponent } from '../shared/avatar/avatar.component';

@Component({
  selector: 'app-player-overlay',
  standalone: true,
  imports: [
    CommonModule,
    MatDialogModule,
    MatIconModule,
    MatButtonModule,
    MatProgressSpinnerModule,
    MatChipsModule,
    TranslatePipe,
    SkeletonComponent,
    AvatarComponent
  ],
  templateUrl: './player-overlay.component.html',
  styleUrls: ['./player-overlay.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class PlayerOverlayComponent implements OnInit {
  player = signal<PlayerDetail | null>(null);
  rivalries = signal<PlayerRivalriesResponse | null>(null);
  loading = signal<boolean>(true);

  constructor(
    private playerService: PlayerService,
    public dialogRef: MatDialogRef<PlayerOverlayComponent>,
    @Inject(MAT_DIALOG_DATA) public data: { playerId: number, ratingType: string }
  ) {}

  ngOnInit(): void {
    this.loadPlayerData();
  }

  loadPlayerData(): void {
    this.loading.set(true);

    this.playerService.getPlayerDetail(this.data.playerId, this.data.ratingType).subscribe({
      next: (player) => {
        this.player.set(player);
        this.loading.set(false);
      },
      error: (err) => {
        console.error('Error loading player:', err);
        this.loading.set(false);
      }
    });

    this.playerService.getPlayerRivalries(this.data.playerId).subscribe({
      next: (res) => this.rivalries.set(res),
      error: (err) => console.error('Error loading rivalries:', err)
    });
  }

  getCueScoreUrl(): string {
    if (!this.player()) return '#';
    return this.player()!.cuescoreProfileUrl;
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

  formatDate(dateString: string | null | undefined): string {
    if (!dateString) return 'N/A';
    const date = new Date(dateString);
    const day = date.getDate().toString().padStart(2, '0');
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const year = date.getFullYear();
    return `${day}/${month}/${year}`;
  }

  close(): void {
    this.dialogRef.close();
  }
}