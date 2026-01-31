import { Component, input, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../pipes/translate.pipe';
import { HeadToHeadMatch } from '../../../models/api';
import { formatDate } from '../../../utils/format.utils';

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
  formatDate = formatDate;
}