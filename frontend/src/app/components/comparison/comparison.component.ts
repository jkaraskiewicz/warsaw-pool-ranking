import { Component, Inject, OnInit, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MAT_DIALOG_DATA, MatDialogRef, MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatChipsModule } from '@angular/material/chips';
import { PlayerService } from '../../services/player.service';
import { HeadToHeadResponse } from '../../models/api';
import { TranslatePipe } from '../../pipes/translate.pipe';
import { HeadToHeadStatsComponent } from './stats/head-to-head-stats.component';
import { MatchHistoryComponent } from './history/match-history.component';

@Component({
  selector: 'app-comparison',
  standalone: true,
  imports: [
    CommonModule,
    MatDialogModule,
    MatIconModule,
    MatButtonModule,
    MatProgressSpinnerModule,
    MatChipsModule,
    TranslatePipe,
    HeadToHeadStatsComponent,
    MatchHistoryComponent
  ],
  templateUrl: './comparison.component.html',
  styleUrls: ['./comparison.component.scss']
})
export class ComparisonComponent implements OnInit {
  comparison = signal<HeadToHeadResponse | null>(null);
  loading = signal<boolean>(true);

  constructor(
    private playerService: PlayerService,
    public dialogRef: MatDialogRef<ComparisonComponent>,
    @Inject(MAT_DIALOG_DATA) public data: { player1Id: number, player2Id: number, ratingType: string }
  ) {}

  ngOnInit() {
    this.loadComparison();
  }

  loadComparison() {
    this.loading.set(true);
    this.playerService.getHeadToHeadComparison(this.data.player1Id, this.data.player2Id, this.data.ratingType)
      .subscribe({
        next: (res) => {
          this.comparison.set(res);
          this.loading.set(false);
        },
        error: (err) => {
          console.error(err);
          this.loading.set(false);
        }
      });
  }

  close() {
    this.dialogRef.close();
  }

  getConfidenceColor(level: string): string {
    switch (level) {
      case 'established': return 'primary';
      case 'emerging': return 'accent';
      case 'provisional': return 'warn';
      default: return '';
    }
  }

  formatProb(prob: number): string {
    return (prob * 100).toFixed(1) + '%';
  }
}