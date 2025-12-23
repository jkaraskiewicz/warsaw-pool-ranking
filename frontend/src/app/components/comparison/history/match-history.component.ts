import { Component, input, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../pipes/translate.pipe';
import { HeadToHeadMatch } from '../../../models/api';

@Component({
  selector: 'app-match-history',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './match-history.component.html',
  styleUrls: ['./match-history.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class MatchHistoryComponent {
  matches = input.required<HeadToHeadMatch[]>();

  formatDate(dateStr: string): string {
    const date = new Date(dateStr);
    const day = date.getDate().toString().padStart(2, '0');
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const year = date.getFullYear();
    return `${day}/${month}/${year}`;
  }
}