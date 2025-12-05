import { Component, EventEmitter, Input, Output } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatButtonToggleModule } from '@angular/material/button-toggle';
import { TranslatePipe } from '../../../pipes/translate.pipe';

@Component({
  selector: 'app-rating-type-selector',
  standalone: true,
  imports: [CommonModule, MatButtonToggleModule, TranslatePipe],
  templateUrl: './rating-type-selector.component.html',
  styleUrls: ['./rating-type-selector.component.scss']
})
export class RatingTypeSelectorComponent {
  @Input() selectedType: string = 'all';
  @Output() typeChange = new EventEmitter<string>();

  ratingTypes = [
    { value: 'all', viewValue: 'ALL_TIME' },
    { value: '1y', viewValue: 'LAST_YEAR' },
    { value: '2y', viewValue: 'LAST_2_YEARS' },
    { value: '3y', viewValue: 'LAST_3_YEARS' },
    { value: '4y', viewValue: 'LAST_4_YEARS' },
    { value: '5y', viewValue: 'LAST_5_YEARS' },
  ];
}