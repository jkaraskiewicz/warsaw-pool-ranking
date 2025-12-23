import { Injectable, signal, computed } from '@angular/core';

const STORAGE_KEY = 'interested_players';

@Injectable({
  providedIn: 'root'
})
export class InterestedPlayersService {
  // Private signal for mutable state
  private _interestedIdsSignal = signal<Set<number>>(new Set());

  // Public readonly signal exposing the interested player IDs
  readonly interestedPlayerIds = computed(() => {
    return new Set(this._interestedIdsSignal()) as ReadonlySet<number>;
  });

  // Computed signal for count (used for badge display)
  readonly count = computed(() => this._interestedIdsSignal().size);

  constructor() {
    this.loadFromStorage();
  }

  private loadFromStorage(): void {
    try {
      const stored = localStorage.getItem(STORAGE_KEY);
      if (stored) {
        const ids = JSON.parse(stored) as number[];
        if (Array.isArray(ids)) {
          this._interestedIdsSignal.set(new Set(ids));
        }
      }
    } catch (e) {
      console.error('Failed to load interested players from storage', e);
      // Fallback to empty set on error
      this._interestedIdsSignal.set(new Set());
    }
  }

  private saveToStorage(ids: Set<number>): void {
    try {
      const array = Array.from(ids);
      localStorage.setItem(STORAGE_KEY, JSON.stringify(array));
    } catch (e) {
      console.error('Failed to save interested players to storage', e);
      // Silently fail - feature degrades to session-only
    }
  }

  toggleInterested(playerId: number): void {
    const current = new Set(this._interestedIdsSignal());

    if (current.has(playerId)) {
      current.delete(playerId);
    } else {
      current.add(playerId);
    }

    this._interestedIdsSignal.set(current);
    this.saveToStorage(current);
  }

  isInterested(playerId: number): boolean {
    return this._interestedIdsSignal().has(playerId);
  }

  clearAll(): void {
    this._interestedIdsSignal.set(new Set());
    try {
      localStorage.removeItem(STORAGE_KEY);
    } catch (e) {
      console.error('Failed to clear interested players from storage', e);
    }
  }
}
