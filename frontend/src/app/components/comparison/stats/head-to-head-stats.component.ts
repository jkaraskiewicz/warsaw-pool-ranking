import { Component, input, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { TranslatePipe } from '../../../pipes/translate.pipe';
import { HeadToHeadResponse } from '../../../models/api';

@Component({
  selector: 'app-head-to-head-stats',
  standalone: true,
  imports: [CommonModule, TranslatePipe],
  templateUrl: './head-to-head-stats.component.html',
  styleUrls: ['./head-to-head-stats.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class HeadToHeadStatsComponent {
  comparison = input.required<HeadToHeadResponse>();
}