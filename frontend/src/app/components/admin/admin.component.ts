import { Component } from '@angular/core';
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
  styleUrls: ['./admin.component.scss']
})
export class AdminComponent {
  password = '';
  loading = false;
  loginLoading = false;

  constructor(
    public adminService: AdminService,
    private snackBar: MatSnackBar
  ) {}

  login(): void {
    if (!this.password) {
      return;
    }

    this.loginLoading = true;
    this.adminService.login(this.password).subscribe({
      next: (result) => {
        this.loginLoading = false;
        if (result.success) {
          this.snackBar.open('Login successful', 'Close', { duration: 2000 });
          this.password = '';
        } else {
          this.snackBar.open(
            result.error || 'Login failed',
            'Close',
            { duration: 4000 }
          );
          this.password = '';
        }
      },
      error: (err) => {
        this.loginLoading = false;
        this.snackBar.open(
          'Login request failed. Please try again.',
          'Close',
          { duration: 4000 }
        );
        console.error('Login error:', err);
        this.password = '';
      }
    });
  }

  logout(): void {
    this.adminService.logout();
  }

  refreshData(): void {
    this.loading = true;
    this.adminService.triggerRefresh().subscribe({
      next: () => {
        this.snackBar.open('Refresh triggered successfully', 'Close', { duration: 3000 });
        this.loading = false;
      },
      error: (err) => {
        if (err.status === 401) {
          this.snackBar.open(
            'Session expired. Please login again.',
            'Close',
            { duration: 4000 }
          );
          this.adminService.logout();
        } else {
          this.snackBar.open(
            'Refresh failed: ' + err.message,
            'Close',
            { duration: 3000 }
          );
        }
        this.loading = false;
      }
    });
  }
}