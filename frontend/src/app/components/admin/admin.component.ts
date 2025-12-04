import { Component } from '@angular/core';
import { AdminService } from '../../services/admin.service';
import { MatSnackBar } from '@angular/material/snack-bar';

@Component({
  selector: 'app-admin',
  templateUrl: './admin.component.html',
  styleUrls: ['./admin.component.scss']
})
export class AdminComponent {
  password = '';
  loading = false;

  constructor(
    public adminService: AdminService,
    private snackBar: MatSnackBar
  ) {}

  login(): void {
    this.adminService.login(this.password);
    this.password = '';
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
        this.snackBar.open('Refresh failed: ' + err.message, 'Close', { duration: 3000 });
        this.loading = false;
      }
    });
  }
}
