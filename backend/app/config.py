"""Application configuration using Pydantic settings."""

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    """Application settings loaded from environment variables."""

    # Database
    database_url: str = "postgresql://user:password@localhost:5432/warsaw_pool_rankings"

    # CueScore API
    cuescore_api_base_url: str = "https://api.cuescore.com"
    cuescore_rate_limit: int = 1  # requests per second

    # Rating Algorithm
    starter_rating: int = 500
    time_decay_half_life_days: int = 1095  # 3 years
    calculation_version: str = "v1"

    # Server
    api_host: str = "0.0.0.0"
    api_port: int = 8000
    debug: bool = False
    cors_origins: list[str] = ["http://localhost:4200", "http://localhost:3000"]

    # Logging
    log_level: str = "INFO"

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
    )


# Global settings instance
settings = Settings()
