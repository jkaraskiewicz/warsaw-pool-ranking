import { Component, EventEmitter, input, Output, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
import { TranslatePipe } from '../../../pipes/translate.pipe';

@Component({
  selector: 'app-rating-type-selector',
  standalone: true,
  imports: [CommonModule, MatButtonToggleModule, TranslatePipe],
  templateUrl: './rating-type-selector.component.html',
  styleUrls: ['./rating-type-selector.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class RatingTypeSelectorComponent {
  selectedType = input<string>('active');
  @Output() typeChange = new EventEmitter<string>();

  ratingTypes = [
    { value: 'active', viewValue: 'ACTIVE' },
    { value: 'all', viewValue: 'ALL_TIME' },
    { value: '1y', viewValue: 'LAST_YEAR' },
    { value: '2y', viewValue: 'LAST_2_YEARS' },
  ];
}
