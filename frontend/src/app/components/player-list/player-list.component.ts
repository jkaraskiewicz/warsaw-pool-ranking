import { Component, OnInit } from '@angular/core';
import { MatDialog } from '@angular/material/dialog';
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
  filteredPlayers: PlayerListItem[] = [];
  searchQuery: string = '';
  loading: boolean = true;
  displayedColumns: string[] = ['rank', 'name', 'rating', 'games', 'confidence', 'change'];

  constructor(
    private playerService: PlayerService,
    private dialog: MatDialog
  ) {}

  ngOnInit(): void {
    this.loadPlayers();
  }

  loadPlayers(): void {
    this.loading = true;
    this.playerService.getPlayers().subscribe({
      next: (data) => {
        this.players = data;
        this.filteredPlayers = data;
        this.loading = false;
      },
      error: (err) => {
        console.error('Error loading players:', err);
        this.loading = false;
      }
    });
  }

  onSearchChange(): void {
    const query = this.searchQuery.toLowerCase().trim();
    if (!query) {
      this.filteredPlayers = this.players;
      return;
    }

    this.filteredPlayers = this.players.filter(player =>
      player.name.toLowerCase().includes(query)
    );
  }

  openPlayerOverlay(player: PlayerListItem): void {
    this.dialog.open(PlayerOverlayComponent, {
      width: '800px',
      maxWidth: '95vw',
      data: { playerId: player.player_id }
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
