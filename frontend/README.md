# Warsaw Pool Rankings - Frontend

Angular frontend for the Warsaw Pool Rankings system.

## Features

- Player rankings table with search functionality
- Player detail overlay with rating history chart
- CueScore profile links
- Material Design UI
- Responsive layout

## Development

### Prerequisites

- Node.js 18+ and npm
- Angular CLI

### Install Dependencies

```bash
npm install
```

### Development Server

```bash
npm start
```

Navigate to `http://localhost:4200/`. The app will automatically reload if you change any source files.

The API proxy is configured to forward `/api/*` requests to `http://localhost:8000` (backend).

### Build

```bash
npm run build
```

The build artifacts will be stored in the `dist/` directory.

### Running Tests

```bash
npm test
```

### Linting

```bash
npm run lint
```

## Project Structure

```
src/
├── app/
│   ├── components/
│   │   ├── player-list/          # Main player rankings table
│   │   ├── player-overlay/       # Player detail dialog
│   │   └── rating-history-chart/ # Chart.js rating history
│   ├── models/
│   │   └── player.model.ts       # TypeScript interfaces
│   ├── services/
│   │   └── player.service.ts     # API communication
│   ├── app.component.*           # Root component
│   └── app.module.ts             # App module
├── assets/                       # Static assets
├── index.html                    # Main HTML
├── main.ts                       # Bootstrap
└── styles.scss                   # Global styles
```

## API Integration

The frontend communicates with the FastAPI backend through these endpoints:

- `GET /api/players?min_games=10` - Get ranked players list
- `GET /api/player/:id` - Get player details
- `GET /api/player/:id/history` - Get rating history snapshots

## Components

### PlayerListComponent
- Displays sortable/searchable table of all players
- Shows rank, name, rating, games, confidence, and recent change
- Clicking a player opens the overlay

### PlayerOverlayComponent
- Modal dialog showing detailed player information
- Displays rating breakdown (ML rating, blending weights)
- Shows rating history chart
- Links to CueScore profile

### RatingHistoryChartComponent
- Line chart showing rating evolution over time
- Uses Chart.js via ng2-charts
- Displays weekly snapshots from simulation

## Styling

- Uses Angular Material with Indigo-Pink theme
- Custom SCSS for component-specific styles
- Responsive design for mobile/desktop
