import { Component, OnInit, Inject, ChangeDetectionStrategy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MAT_DIALOG_DATA, MatDialogRef, MatDialogModule } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatChipsModule } from '@angular/material/chips';
import { MatTooltipModule } from '@angular/material/tooltip';
import { PlayerService } from '../../services/player.service';
import { TranslationService } from '../../services/translation.service';
import { PlayerDetail, PlayerRivalriesResponse } from '../../models/api';
import { getConfidenceColor, formatDate } from '../../utils/format.utils';
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
    MatTooltipModule,
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
    private translationService: TranslationService,
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

  getConfidenceColor = getConfidenceColor;
  formatDate = formatDate;

  getRatingInterpretation(rating: number): string {
    if (rating >= 2000) return this.translationService.translate('RATING_EXPERT');
    if (rating >= 1800) return this.translationService.translate('RATING_ADVANCED');
    if (rating >= 1600) return this.translationService.translate('RATING_INTERMEDIATE');
    if (rating >= 1400) return this.translationService.translate('RATING_DEVELOPING');
    return this.translationService.translate('RATING_BEGINNER');
  }

  getConfidenceLabel(level: string): string {
    const key = `CONFIDENCE_${level.toUpperCase()}`;
    return this.translationService.translate(key);
  }

  getConfidenceTooltip(level: string, gamesPlayed: number): string {
    switch (level) {
      case 'established':
        return this.translationService.translate('CONFIDENCE_ESTABLISHED_TOOLTIP');
      case 'emerging':
        return this.translationService.translate('CONFIDENCE_EMERGING_TOOLTIP');
      case 'provisional':
        return this.translationService.translate('CONFIDENCE_PROVISIONAL_TOOLTIP');
      default:
        return this.translationService.translate('CONFIDENCE_UNRANKED_TOOLTIP');
    }
  }

  getGamesContext(effectiveGames: number): string {
    const recentGames = Math.round(effectiveGames);
    if (recentGames >= 40) return this.translationService.translate('GAMES_VERY_ACTIVE');
    if (recentGames >= 20) return this.translationService.translate('GAMES_ACTIVE');
    if (recentGames >= 10) return this.translationService.translate('GAMES_MODERATE');
    return this.translationService.translate('GAMES_LIMITED');
  }

  close(): void {
    this.dialogRef.close();
  }
}