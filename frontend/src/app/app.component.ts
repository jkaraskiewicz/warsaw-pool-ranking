import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatMenuModule } from '@angular/material/menu';
import { TranslatePipe } from './pipes/translate.pipe';
import { ThemeService } from './services/theme.service';
import { TranslationService } from './services/translation.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    CommonModule,
    RouterModule,
    MatToolbarModule,
    MatIconModule,
    MatButtonModule,
    MatMenuModule,
    TranslatePipe
  ],
  templateUrl: './app.component.html',
  styleUrls: ['./app.component.scss']
})
export class AppComponent {
  title = 'WARSAW_POOL_RANKINGS';

  isDarkMode = this.themeService.isDarkMode;
  currentLang = this.translationService.currentLang;

  constructor(
    private themeService: ThemeService,
    private translationService: TranslationService
  ) {}

  toggleTheme() {
    this.themeService.toggleTheme();
  }

  setLanguage(lang: 'en' | 'pl') {
    this.translationService.setLanguage(lang);
  }
}