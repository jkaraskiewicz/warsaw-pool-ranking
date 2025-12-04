import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root'
})
export class AdminService {
  private apiUrl = '/api/admin';
  private tokenKey = 'admin_token';

  constructor(private http: HttpClient) {}

  login(password: string): void {
    // Simple "login" by saving the token directly. 
    // In this simplified scheme, the password serves as the Bearer token.
    localStorage.setItem(this.tokenKey, password);
  }

  logout(): void {
    localStorage.removeItem(this.tokenKey);
  }

  isLoggedIn(): boolean {
    return !!localStorage.getItem(this.tokenKey);
  }

  triggerRefresh(): Observable<any> {
    const token = localStorage.getItem(this.tokenKey);
    const headers = new HttpHeaders().set('Authorization', `Bearer ${token}`);
    return this.http.post(`${this.apiUrl}/refresh`, {}, { headers });
  }
}
