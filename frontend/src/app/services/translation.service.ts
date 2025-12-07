import { Injectable, signal, computed } from '@angular/core';
import { DICTIONARY } from '../i18n/dictionary';

export type Language = 'en' | 'pl';

@Injectable({
  providedIn: 'root'
})
export class TranslationService {
  private currentLangSignal = signal<Language>('en');

  // Public signal for the current language
  currentLang = this.currentLangSignal.asReadonly();

  constructor() {
    const savedLang = localStorage.getItem('lang') as Language;
    if (savedLang === 'pl' || savedLang === 'en') {
      this.setLanguage(savedLang);
    }
  }

  setLanguage(lang: Language) {
    this.currentLangSignal.set(lang);
    localStorage.setItem('lang', lang);
  }

  translate(key: string): string {
    const lang = this.currentLangSignal();
    const dict = DICTIONARY[lang];
    const normalizedKey = key.toUpperCase();
    return (dict as any)[normalizedKey] || (dict as any)[key] || key;
  }
}