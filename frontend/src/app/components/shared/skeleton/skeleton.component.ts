import { Component, input, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-skeleton',
  standalone: true,
  imports: [CommonModule],
  template: `<div class="skeleton" [style.width]="width()" [style.height]="height()" [style.border-radius]="borderRadius()"></div>`,
  styles: [`
    .skeleton {
      background-color: rgba(255, 255, 255, 0.1);
      animation: pulse 1.5s infinite ease-in-out;
    }
    @keyframes pulse {
      0% { opacity: 0.6; }
      50% { opacity: 0.3; }
      100% { opacity: 0.6; }
    }
  `],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class SkeletonComponent {
  width = input<string>('100%');
  height = input<string>('1rem');
  borderRadius = input<string>('4px');
}
