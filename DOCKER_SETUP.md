# Docker Setup Guide

This guide explains how to run the Warsaw Pool Rankings application using Docker and docker-compose.

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+

## Quick Start

### 1. Configure Venues

Edit `backend/config/venues.py` to add Warsaw pool venues you want to track:

```python
WARSAW_VENUES = [
    {
        "id": "67496954",
        "slug": "147-break-nowogrodzka",
        "name": "147 Break Nowogrodzka"
    },
    # Add more venues here
]
```

### 2. Set Environment Variables (Optional)

Copy `.env.example` to `.env` and customize if needed:

```bash
cp .env.example .env
```

Default password is `poolpass123` - change in production!

### 3. Build and Start Services

```bash
# Build all containers
docker-compose build

# Start all services
docker-compose up -d
```

This will start:
- **PostgreSQL** (port 5432) - Database
- **Backend** (port 8000) - FastAPI API
- **Frontend** (port 80) - Angular + Nginx

### 4. Initialize Database with Data

The first time you run the application, you need to populate the database:

```bash
# Run initialization script inside the backend container
docker-compose exec backend python scripts/init_database.py --auto
```

This will:
1. Create all database tables
2. Scrape tournament IDs from configured venues
3. Fetch tournament data from CueScore API (respects 1 req/sec rate limit)
4. Calculate ratings for all players
5. Populate the database

**Expected time:** 10-30 minutes depending on number of tournaments.

### 5. Access the Application

- **Frontend:** http://localhost
- **Backend API:** http://localhost:8000
- **API Docs:** http://localhost:8000/docs

## Docker Services

### Database (PostgreSQL)
```yaml
Container: warsaw-pool-db
Port: 5432
Volume: postgres_data (persistent)
```

### Backend (FastAPI)
```yaml
Container: warsaw-pool-backend
Port: 8000
Hot Reload: Enabled (mounted volume)
```

### Frontend (Angular + Nginx)
```yaml
Container: warsaw-pool-frontend
Port: 80
Serves: Production build
```

## Common Commands

### View Logs
```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f backend
docker-compose logs -f frontend
docker-compose logs -f db
```

### Stop Services
```bash
# Stop all services
docker-compose down

# Stop and remove volumes (DELETES DATABASE!)
docker-compose down -v
```

### Restart Services
```bash
# Restart all
docker-compose restart

# Restart specific service
docker-compose restart backend
```

### Run Commands in Containers
```bash
# Backend shell
docker-compose exec backend bash

# Database shell
docker-compose exec db psql -U pool_app -d warsaw_pool_rankings

# Run tests
docker-compose exec backend pytest

# Weekly update
docker-compose exec backend python scripts/run_weekly_update.py
```

### Database Operations
```bash
# Connect to database
docker-compose exec db psql -U pool_app -d warsaw_pool_rankings

# Check player count
docker-compose exec db psql -U pool_app -d warsaw_pool_rankings -c "SELECT COUNT(*) FROM players;"

# View top 10 players
docker-compose exec db psql -U pool_app -d warsaw_pool_rankings -c "
SELECT p.name, r.rating, r.games_played, r.confidence_level
FROM players p
JOIN ratings r ON p.id = r.player_id
ORDER BY r.rating DESC
LIMIT 10;"
```

### Rebuild After Code Changes
```bash
# Rebuild specific service
docker-compose build backend
docker-compose build frontend

# Rebuild and restart
docker-compose up -d --build
```

## Development Workflow

### Backend Development
The backend container mounts `./backend` as a volume with hot reload enabled:

1. Edit code in `backend/` directory
2. FastAPI automatically reloads
3. Changes are reflected immediately

### Frontend Development
For frontend development, you may want to run locally instead:

```bash
# Stop Docker frontend
docker-compose stop frontend

# Run locally with hot reload
cd frontend
npm install
npm start
# Access at http://localhost:4200
```

## Weekly Data Updates

Schedule weekly updates using cron:

```bash
# Add to crontab
0 2 * * 0 docker-compose exec -T backend python scripts/run_weekly_update.py
```

Or run manually:
```bash
docker-compose exec backend python scripts/run_weekly_update.py
```

## Production Deployment

### Security Checklist
- [ ] Change `POSTGRES_PASSWORD` in `.env`
- [ ] Use environment-specific `.env` file
- [ ] Enable HTTPS (add reverse proxy like Traefik/Nginx)
- [ ] Set up automated backups for `postgres_data` volume
- [ ] Configure firewall rules
- [ ] Set up monitoring and alerting

### Database Backups
```bash
# Backup
docker-compose exec db pg_dump -U pool_app warsaw_pool_rankings > backup.sql

# Restore
docker-compose exec -T db psql -U pool_app -d warsaw_pool_rankings < backup.sql
```

### Volume Backups
```bash
# Backup postgres volume
docker run --rm \
  -v warsaw-pool-ranking_postgres_data:/data \
  -v $(pwd):/backup \
  alpine tar czf /backup/postgres-backup.tar.gz -C /data .

# Restore
docker run --rm \
  -v warsaw-pool-ranking_postgres_data:/data \
  -v $(pwd):/backup \
  alpine tar xzf /backup/postgres-backup.tar.gz -C /data
```

## Troubleshooting

### Port Conflicts
If port 80 or 8000 is already in use:

```yaml
# Edit docker-compose.yml
services:
  frontend:
    ports:
      - "8080:80"  # Change to 8080
  backend:
    ports:
      - "8001:8000"  # Change to 8001
```

### Database Connection Issues
```bash
# Check database is running
docker-compose ps db

# Check database logs
docker-compose logs db

# Test connection
docker-compose exec backend python -c "from app.database import get_engine; get_engine().connect()"
```

### Reset Everything
```bash
# Stop and remove all containers, networks, and volumes
docker-compose down -v

# Remove images
docker-compose down --rmi all

# Start fresh
docker-compose up -d
docker-compose exec backend python scripts/init_database.py --auto
```

### Check Service Health
```bash
# Check all services
docker-compose ps

# Check specific service
docker-compose exec backend curl http://localhost:8000/health
```

## Architecture

```
┌─────────────┐
│   Browser   │
└──────┬──────┘
       │ HTTP :80
       ▼
┌─────────────────┐
│  Nginx          │
│  (Frontend)     │◄────┐
└────────┬────────┘     │
         │              │ API Proxy
         │              │ /api/*
         ▼              │
    Static Files        │
    (Angular Build)     │
                        │
         ┌──────────────┘
         │
         ▼
┌─────────────────┐
│  FastAPI        │
│  (Backend)      │
│  Port: 8000     │
└────────┬────────┘
         │
         │ SQL
         ▼
┌─────────────────┐
│  PostgreSQL     │
│  Port: 5432     │
│  Volume: data   │
└─────────────────┘
```
