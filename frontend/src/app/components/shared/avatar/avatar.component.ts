import { Component, computed, signal, ChangeDetectionStrategy, input, effect } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-avatar',
  standalone: true,
  imports: [CommonModule],
  template: `
    <div class="avatar-container" [ngClass]="size()" [style.font-size]="fontSize()">
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
export class AvatarComponent {
  // Modern Angular 17 signal inputs
  playerId = input<number | undefined>(undefined);
  name = input<string | undefined | null>(null);
  size = input<'small' | 'medium' | 'large' | 'xlarge'>('small');

  imageLoaded = signal(false);
  imageError = signal(false);

  initial = computed(() => {
    const currentName = this.name();
    return currentName ? currentName.charAt(0).toUpperCase() : '?';
  });

  avatarUrl = computed(() => {
    const id = this.playerId();
    if (!id) return null;
    // Map size to API size if needed, but for now assuming direct mapping
    // or we can just use the size passed.
    // However, existing code used 'medium' url for 'large' visual in overlay.
    // Let's assume the API accepts 'small', 'medium', 'large'.
    // If size is 'xlarge', we might want to request 'large' from API.
    const currentSize = this.size();
    const apiSize = currentSize === 'xlarge' ? 'large' : currentSize;
    return `/api/avatars/${id}/${apiSize}`;
  });

  fontSize = computed(() => {
    switch (this.size()) {
      case 'small': return 'var(--text-sm)'; // 0.875rem
      case 'medium': return '1.5rem';
      case 'large': return '2.5rem';
      case 'xlarge': return '3.5rem';
      default: return '1rem';
    }
  });

  constructor() {
    // Reset image state when playerId changes
    effect(() => {
      this.playerId(); // Track playerId changes
      this.imageLoaded.set(false);
      this.imageError.set(false);
    });
  }

  onLoad() {
    this.imageLoaded.set(true);
  }

  onError() {
    this.imageError.set(true);
    this.imageLoaded.set(false);
  }
}
