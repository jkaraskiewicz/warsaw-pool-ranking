import { Routes } from '@angular/router';
import { PlayerListComponent } from './components/player-list/player-list.component';
import { AdminComponent } from './components/admin/admin.component';

export const routes: Routes = [
  { path: '', component: PlayerListComponent },
  { path: 'admin', component: AdminComponent },
  { path: '**', redirectTo: '' }
];
