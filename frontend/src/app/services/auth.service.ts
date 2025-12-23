import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, map, catchError, of } from 'rxjs';

interface LoginResponse {
  success: boolean;
  message?: string;
}

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private tokenKey = 'admin_token';
  private apiUrl = '/api/admin';

  constructor(private http: HttpClient) {}

  login(password: string): Observable<{ success: boolean; error?: string }> {
    return this.http.post<LoginResponse>(`${this.apiUrl}/login`, { password })
      .pipe(
        map(response => {
          if (response.success) {
            localStorage.setItem(this.tokenKey, password);
            return { success: true };
          } else {
            return {
              success: false,
              error: response.message || 'Invalid password'
            };
          }
        }),
        catchError(error => {
          console.error('Login request failed:', error);
          return of({
            success: false,
            error: 'Network error. Please try again.'
          });
        })
      );
  }

  logout(): void {
    localStorage.removeItem(this.tokenKey);
  }

  isLoggedIn(): boolean {
    return !!localStorage.getItem(this.tokenKey);
  }

  getToken(): string | null {
    return localStorage.getItem(this.tokenKey);
  }
}
