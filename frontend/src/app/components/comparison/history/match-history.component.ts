import { Component, Input } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatTableModule } from '@angular/material/table';
import { TranslatePipe } from '../../../pipes/translate.pipe';
import { HeadToHeadMatch } from '../../../models/api';

@Component({
  selector: 'app-match-history',
  standalone: true,
  imports: [CommonModule, MatTableModule, TranslatePipe],
  templateUrl: './match-history.component.html',
  styleUrls: ['./match-history.component.scss']
})
export class MatchHistoryComponent {
  @Input({ required: true }) matches!: HeadToHeadMatch[];
  displayedColumns: string[] = ['date', 'tournament', 'score'];

  formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString();
  }
}
