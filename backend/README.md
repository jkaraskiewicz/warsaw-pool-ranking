# Warsaw Pool Rankings - Backend

Python FastAPI backend for the Warsaw Pool Ranking System.

## Setup

### Prerequisites
- Python 3.11+
- PostgreSQL 14+

### Installation

1. Create virtual environment:
```bash
python -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate
```

2. Install dependencies:
```bash
pip install -r requirements.txt
```

3. Set up environment variables:
```bash
cp .env.example .env
# Edit .env with your database credentials
```

4. Set up database:
```bash
# Create database
createdb warsaw_pool_rankings

# Run schema
psql warsaw_pool_rankings < ../database/schema.sql
```

### Running the Server

Development mode:
```bash
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

Production mode:
```bash
uvicorn app.main:app --host 0.0.0.0 --port 8000
```

API will be available at: http://localhost:8000

API documentation: http://localhost:8000/docs

## Project Structure

```
backend/
├── app/
│   ├── api/           # API endpoints
│   ├── rating/        # Rating calculation modules
│   ├── data/          # CueScore data collection
│   ├── models.py      # SQLAlchemy models
│   ├── database.py    # Database connection
│   ├── config.py      # Configuration
│   └── main.py        # FastAPI app
├── tests/             # Unit tests
├── requirements.txt   # Python dependencies
└── .env              # Environment variables
```

## Development

### Running Tests
```bash
pytest
```

### Code Formatting
```bash
black app/
isort app/
```

### Type Checking
```bash
mypy app/
```

## API Endpoints

- `GET /` - API health check
- `GET /health` - Health check for monitoring
- `GET /api/players` - List all players (coming soon)
- `GET /api/player/:id` - Player details (coming soon)
- `GET /api/player/:id/history` - Player rating history (coming soon)
