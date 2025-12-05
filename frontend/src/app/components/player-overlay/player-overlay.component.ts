import { Component, OnInit, Inject } from '@angular/core';
import { MAT_DIALOG_DATA, MatDialogRef } from '@angular/material/dialog';
import { PlayerService } from '../../services/player.service';
import { PlayerDetail } from '../../models/player.model';

@Component({
  selector: 'app-player-overlay',
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
    return `https://cuescore.com/player/${this.player.cuescoreId}`;
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

  formatDate(dateString: string | null): string {
    if (!dateString) return 'N/A';
    const date = new Date(dateString);
    return date.toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
  }

  close(): void {
    this.dialogRef.close();
  }
}
