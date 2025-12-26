import { Component, Input, OnChanges, SimpleChanges, computed, signal, ChangeDetectionStrategy } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-avatar',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="avatar-container" [ngClass]="size" [style.font-size]="fontSize()">
      <div class="initial">{{ initial() }}</div>
      <img
        *ngIf="avatarUrl()"
        [src]="avatarUrl()"
        (load)="onLoad()"
        (error)="onError()"
        class="avatar-img"
        [class.loaded]="imageLoaded()"
        alt="Avatar">
    </div>
  `,
  styleUrls: ['./avatar.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class AvatarComponent implements OnChanges {
  @Input() playerId: number | undefined;
  @Input() name: string | undefined | null;
  @Input() size: 'small' | 'medium' | 'large' | 'xlarge' = 'small';

  imageLoaded = signal(false);
  imageError = signal(false);

  initial = computed(() => {
    return this.name ? this.name.charAt(0).toUpperCase() : '?';
  });

  avatarUrl = computed(() => {
    if (!this.playerId) return null;
    // Map size to API size if needed, but for now assuming direct mapping
    // or we can just use the size passed.
    // However, existing code used 'medium' url for 'large' visual in overlay.
    // Let's assume the API accepts 'small', 'medium', 'large'.
    // If size is 'xlarge', we might want to request 'large' from API.
    const apiSize = this.size === 'xlarge' ? 'large' : this.size;
    return `/api/avatars/${this.playerId}/${apiSize}`;
  });

  fontSize = computed(() => {
    switch (this.size) {
      case 'small': return 'var(--text-sm)'; // 0.875rem
      case 'medium': return '1.5rem';
      case 'large': return '2.5rem';
      case 'xlarge': return '3.5rem';
      default: return '1rem';
    }
  });

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['playerId']) {
      this.imageLoaded.set(false);
      this.imageError.set(false);
    }
  }

  onLoad() {
    this.imageLoaded.set(true);
  }

  onError() {
    this.imageError.set(true);
    this.imageLoaded.set(false);
  }
}
