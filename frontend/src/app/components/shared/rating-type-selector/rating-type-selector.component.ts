import { Component, input, output, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
import { MatIconModule } from '@angular/material/icon';
import { TranslatePipe } from '../../../pipes/translate.pipe';

@Component({
  selector: 'app-rating-type-selector',
  standalone: true,
  imports: [CommonModule, MatButtonToggleModule, MatIconModule, TranslatePipe],
  templateUrl: './rating-type-selector.component.html',
  styleUrls: ['./rating-type-selector.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class RatingTypeSelectorComponent {
  selectedType = input<string>('active');
  typeChange = output<string>();

  ratingTypes = [
    { value: 'active', viewValue: 'ACTIVE' },
    { value: 'all', viewValue: 'ALL_TIME' },
    { value: '1y', viewValue: 'LAST_YEAR' },
    { value: '2y', viewValue: 'LAST_2_YEARS' },
  ];
}
