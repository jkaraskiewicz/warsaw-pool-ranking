# Warsaw Pool Rankings - Makefile
# Convenience commands for Docker operations

.PHONY: help build up down logs test clean backup tournaments rankings avatars database shell-backend status restart rebuild

help: ## Show this help message
	@echo "Warsaw Pool Rankings - Docker Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

build: ## Build all Docker containers
	docker-compose build

up: ## Start all services
	docker-compose up -d
	@echo ""
	@echo "Services started!"
	@echo "Frontend: http://localhost"
	@echo "Backend:  http://localhost:8000"

down: ## Stop all services
	docker-compose down

logs: ## Show logs from all services
	docker-compose logs -f

# Resource-based CLI commands
tournaments: ## Fetch tournament data from CueScore
	docker-compose exec backend ./warsaw_pool_ranking tournaments refresh

rankings: ## Calculate player ratings
	docker-compose exec backend ./warsaw_pool_ranking rankings refresh

avatars: ## Download/update player avatars
	docker-compose exec backend ./warsaw_pool_ranking avatars refresh

avatars-stats: ## Show avatar storage statistics
	docker-compose exec backend ./warsaw_pool_ranking avatars stats

database-backup: ## Backup SQLite database to file
	@TIMESTAMP=$$(date +%Y%m%d-%H%M%S) && \
	docker-compose exec backend cp /app/data/warsaw_pool_ranking.db /app/data/backup-$$TIMESTAMP.db && \
	echo "Backup created: backend/data/backup-$$TIMESTAMP.db"

database-stats: ## Show database statistics
	@echo "Database Statistics:"
	@echo ""
	@echo -n "Players: "
	@docker-compose exec backend sqlite3 /app/data/warsaw_pool_ranking.db "SELECT COUNT(*) FROM players;"
	@echo -n "Games: "
	@docker-compose exec backend sqlite3 /app/data/warsaw_pool_ranking.db "SELECT COUNT(*) FROM games;"
	@echo -n "Tournaments: "
	@docker-compose exec backend sqlite3 /app/data/warsaw_pool_ranking.db "SELECT COUNT(*) FROM tournaments;"
	@echo -n "Ratings: "
	@docker-compose exec backend sqlite3 /app/data/warsaw_pool_ranking.db "SELECT COUNT(*) FROM ratings;"

players: ## Show top 10 players
	@docker-compose exec backend sqlite3 -header -column /app/data/warsaw_pool_ranking.db "\
	SELECT p.name, r.rating, r.games_played, r.confidence_level \
	FROM players p \
	JOIN ratings r ON p.id = r.player_id \
	WHERE r.rating_type = 'active' \
	ORDER BY r.rating DESC \
	LIMIT 10;"

test: ## Run backend tests
	docker-compose exec backend cargo test

shell-backend: ## Open shell in backend container
	docker-compose exec backend bash

status: ## Show status of all services
	docker-compose ps

restart: ## Restart all services
	docker-compose restart

rebuild: ## Rebuild and restart all services
	docker-compose up -d --build

clean: ## Stop and remove all containers and volumes (WARNING: deletes database!)
	docker-compose down -v
