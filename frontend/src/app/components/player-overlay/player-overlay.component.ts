import { Component, OnInit, Inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MAT_DIALOG_DATA, MatDialogRef, MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatChipsModule } from '@angular/material/chips';
import { PlayerService } from '../../services/player.service';
import { PlayerDetail } from '../../models/api';
import { TranslatePipe } from '../../pipes/translate.pipe';

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
    TranslatePipe
  ],
  templateUrl: './player-overlay.component.html',
  styleUrls: ['./player-overlay.component.scss']
})
export class PlayerOverlayComponent implements OnInit {
  player: PlayerDetail | null = null;
  loading: boolean = true;

  constructor(
    private playerService: PlayerService,
    public dialogRef: MatDialogRef<PlayerOverlayComponent>,
    @Inject(MAT_DIALOG_DATA) public data: { playerId: number, ratingType: string }
  ) {}

  ngOnInit(): void {
    this.loadPlayerData();
  }

  loadPlayerData(): void {
    this.loading = true;

    this.playerService.getPlayerDetail(this.data.playerId, this.data.ratingType).subscribe({
      next: (player) => {
        this.player = player;
        this.loading = false;
      },
      error: (err) => {
        console.error('Error loading player:', err);
        this.loading = false;
      }
    });
  }

  getCueScoreUrl(): string {
    if (!this.player) return '#';
    return this.player.cuescoreProfileUrl;
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

  getAvatarUrl(playerId: number | undefined, size: 'small' | 'medium' | 'large'): string {
    if (!playerId) return 'none';
    return `url(/api/avatars/${playerId}/${size})`;
  }

  close(): void {
    this.dialogRef.close();
  }
}