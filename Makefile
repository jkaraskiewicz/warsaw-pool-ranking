# Warsaw Pool Rankings - Makefile
# Convenience commands for Docker operations

.PHONY: help build up down logs init test clean backup

help: ## Show this help message
	@echo "Warsaw Pool Rankings - Docker Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build all Docker containers
	docker-compose build

up: ## Start all services
	docker-compose up -d
	@echo ""
	@echo "Services started!"
	@echo "Frontend: http://localhost"
	@echo "Backend:  http://localhost:8000"
	@echo "API Docs: http://localhost:8000/docs"

down: ## Stop all services
	docker-compose down

logs: ## Show logs from all services
	docker-compose logs -f

init: ## Initialize database with data (run after first startup)
	@echo "Initializing database with tournament data..."
	@echo "This may take 10-30 minutes..."
	docker-compose exec backend python scripts/init_database.py --auto

update: ## Run weekly data update
	docker-compose exec backend python scripts/run_weekly_update.py

test: ## Run backend tests
	docker-compose exec backend pytest -v

shell-backend: ## Open shell in backend container
	docker-compose exec backend bash

shell-db: ## Open PostgreSQL shell
	docker-compose exec db psql -U pool_app -d warsaw_pool_rankings

status: ## Show status of all services
	docker-compose ps

restart: ## Restart all services
	docker-compose restart

rebuild: ## Rebuild and restart all services
	docker-compose up -d --build

clean: ## Stop and remove all containers and volumes (WARNING: deletes database!)
	docker-compose down -v

backup: ## Backup database to backup.sql
	docker-compose exec db pg_dump -U pool_app warsaw_pool_rankings > backup-$$(date +%Y%m%d-%H%M%S).sql
	@echo "Backup created: backup-$$(date +%Y%m%d-%H%M%S).sql"

players: ## Show top 10 players
	@docker-compose exec db psql -U pool_app -d warsaw_pool_rankings -c "\
	SELECT p.name, r.rating, r.games_played, r.confidence_level \
	FROM players p \
	JOIN ratings r ON p.id = r.player_id \
	ORDER BY r.rating DESC \
	LIMIT 10;"

stats: ## Show database statistics
	@echo "Database Statistics:"
	@echo ""
	@echo -n "Players: "
	@docker-compose exec db psql -U pool_app -d warsaw_pool_rankings -t -c "SELECT COUNT(*) FROM players;"
	@echo -n "Games: "
	@docker-compose exec db psql -U pool_app -d warsaw_pool_rankings -t -c "SELECT COUNT(*) FROM games;"
	@echo -n "Tournaments: "
	@docker-compose exec db psql -U pool_app -d warsaw_pool_rankings -t -c "SELECT COUNT(*) FROM tournaments;"
	@echo -n "Venues: "
	@docker-compose exec db psql -U pool_app -d warsaw_pool_rankings -t -c "SELECT COUNT(*) FROM venues;"
