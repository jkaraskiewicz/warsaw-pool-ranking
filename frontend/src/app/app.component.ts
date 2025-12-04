import { Component } from '@angular/core';
import { ThemeService } from './services/theme.service';
import { TranslationService, Language } from './services/translation.service';

@Component({
  selector: 'app-root',
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent {
  title = 'WARSAW_POOL_RANKINGS';
  isDarkMode = false;
  currentLang: Language = 'en';

  constructor(
    private themeService: ThemeService,
    private translationService: TranslationService
  ) {
    this.themeService.isDarkMode$.subscribe(isDark => this.isDarkMode = isDark);
    this.translationService.currentLang$.subscribe(lang => this.currentLang = lang);
  }

  toggleTheme() {
    this.themeService.toggleTheme();
  }

  toggleLanguage() {
    const newLang = this.currentLang === 'en' ? 'pl' : 'en';
    this.translationService.setLanguage(newLang);
  }
}
