"""FastAPI main application."""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.config import settings

# Create FastAPI app
app = FastAPI(
    title="Warsaw Pool Rankings API",
    description="Backend API for Warsaw Pool Ranking System",
    version="1.0.0",
    debug=settings.debug,
)

# Configure CORS for Angular frontend
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/")
def root():
    """Root endpoint - API health check."""
    return {
        "message": "Warsaw Pool Rankings API",
        "version": "1.0.0",
        "status": "operational",
    }


@app.get("/health")
def health_check():
    """Health check endpoint for monitoring."""
    return {"status": "healthy"}


# Import and include API routers
from app.api import players

app.include_router(players.router, prefix="/api", tags=["players"])
