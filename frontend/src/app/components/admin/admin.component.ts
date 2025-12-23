import { Component, ChangeDetectionStrategy, signal } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar } from '@angular/material/snack-bar';
import { AdminService } from '../../services/admin.service';
import { TranslationService } from '../../services/translation.service';
import { TranslatePipe } from '../../pipes/translate.pipe';

@Component({
  selector: 'app-admin',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
    TranslatePipe
  ],
  templateUrl: './admin.component.html',
  styleUrls: ['./admin.component.scss'],
  changeDetection: ChangeDetectionStrategy.OnPush
})
export class AdminComponent {
  password = signal('');
  loading = signal(false);
  loginLoading = signal(false);

  constructor(
    public adminService: AdminService,
    private snackBar: MatSnackBar,
    private translationService: TranslationService
  ) {}

  login(): void {
    if (!this.password()) {
      return;
    }

    this.loginLoading.set(true);
    this.adminService.login(this.password()).subscribe({
      next: (result) => {
        this.loginLoading.set(false);
        if (result.success) {
          this.snackBar.open(
            this.translationService.translate('LOGIN_SUCCESSFUL'),
            this.translationService.translate('CLOSE'),
            { duration: 2000 }
          );
          this.password.set('');
        } else {
          this.snackBar.open(
            result.error || this.translationService.translate('LOGIN_FAILED'),
            this.translationService.translate('CLOSE'),
            { duration: 4000 }
          );
          this.password.set('');
        }
      },
      error: (err) => {
        this.loginLoading.set(false);
        this.snackBar.open(
          this.translationService.translate('LOGIN_REQUEST_FAILED'),
          this.translationService.translate('CLOSE'),
          { duration: 4000 }
        );
        console.error('Login error:', err);
        this.password.set('');
      }
    });
  }

  logout(): void {
    this.adminService.logout();
  }

  refreshData(): void {
    this.loading.set(true);
    this.adminService.triggerRefresh().subscribe({
      next: () => {
        this.snackBar.open(
          this.translationService.translate('REFRESH_SUCCESSFUL'),
          this.translationService.translate('CLOSE'),
          { duration: 3000 }
        );
        this.loading.set(false);
      },
      error: (err) => {
        if (err.status === 401) {
          this.snackBar.open(
            this.translationService.translate('SESSION_EXPIRED'),
            this.translationService.translate('CLOSE'),
            { duration: 4000 }
          );
          this.adminService.logout();
        } else {
          this.snackBar.open(
            this.translationService.translate('REFRESH_FAILED') + ': ' + err.message,
            this.translationService.translate('CLOSE'),
            { duration: 3000 }
          );
        }
        this.loading.set(false);
      }
    });
  }
}
