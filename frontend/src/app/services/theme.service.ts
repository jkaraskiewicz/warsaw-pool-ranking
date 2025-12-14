import { Injectable, signal } from '@angular/core';

@Injectable({
  providedIn: 'root'
})
export class ThemeService {
  private darkModeSignal = signal<boolean>(false);
  isDarkMode = this.darkModeSignal.asReadonly();

  constructor() {
    const savedTheme = localStorage.getItem('theme');
    if (savedTheme === 'dark') {
      this.setDarkMode(true);
    } else {
      this.setDarkMode(false);
    }
  }

  setDarkMode(isDark: boolean) {
    this.darkModeSignal.set(isDark);
    const body = document.body;

    if (isDark) {
      body.classList.remove('light-theme');
      localStorage.setItem('theme', 'dark');
    } else {
      body.classList.add('light-theme');
      localStorage.setItem('theme', 'light');
    }
  }

  toggleTheme() {
    this.setDarkMode(!this.darkModeSignal());
  }
}