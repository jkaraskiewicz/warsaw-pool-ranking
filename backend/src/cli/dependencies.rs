use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::cache::Cache;
use crate::config::paths;
use crate::config::settings::AppConfig;
use crate::database;
use crate::fetchers::cuescore_models::TournamentResponse;
use crate::services::ingestion::IngestionService;
use crate::services::processing::ProcessingService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Resource {
    Tournaments,
    Rankings,
    Avatars,
    Database,
}

pub struct DependencyGraph {
    dependencies: HashMap<Resource, Vec<Resource>>,
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    pub fn new() -> Self {
        let mut dependencies = HashMap::new();
        // Rankings depends on tournaments
        dependencies.insert(Resource::Rankings, vec![Resource::Tournaments]);
        // Avatars depend on rankings (which provide player data)
        dependencies.insert(Resource::Avatars, vec![Resource::Rankings]);
        Self { dependencies }
    }

    /// Check if resource exists in the system
    fn resource_exists(&self, resource: &Resource) -> Result<bool> {
        match resource {
            Resource::Tournaments => {
                // Check if cache/parsed/tournaments.json exists and has data
                let cache = Cache::new(paths::get_cache_dir())?;
                Ok(cache.load_parsed::<Vec<TournamentResponse>>("tournaments")?.is_some())
            },
            Resource::Rankings => {
                // Check if database exists and has ratings
                let db_path = paths::get_database_path();
                if !Path::new(&db_path).exists() {
                    return Ok(false);
                }
                let pool = database::create_pool(&db_path)?;
                let conn = pool.get()?;
                let count: i64 = conn.query_row("SELECT COUNT(*) FROM ratings", [], |row| row.get(0))?;
                Ok(count > 0)
            },
            Resource::Avatars => Ok(true), // Avatars are optional
            Resource::Database => Ok(Path::new(&paths::get_database_path()).exists()),
        }
    }

    /// Get execution plan: ordered list of resources to refresh
    pub fn execution_plan(&self, resource: Resource) -> Result<Vec<Resource>> {
        let mut plan = Vec::new();
        let mut visited = HashSet::new();
        self.build_plan(resource, &mut plan, &mut visited)?;
        Ok(plan)
    }

    fn build_plan(
        &self,
        resource: Resource,
        plan: &mut Vec<Resource>,
        visited: &mut HashSet<Resource>,
    ) -> Result<()> {
        if visited.contains(&resource) {
            return Ok(());
        }
        visited.insert(resource);

        // Add dependencies first (if they don't exist)
        if let Some(deps) = self.dependencies.get(&resource) {
            for dep in deps {
                if !self.resource_exists(dep)? {
                    self.build_plan(*dep, plan, visited)?;
                }
            }
        }

        plan.push(resource);
        Ok(())
    }
}

pub struct OperationOrchestrator {
    graph: DependencyGraph,
}

impl Default for OperationOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationOrchestrator {
    pub fn new() -> Self {
        Self {
            graph: DependencyGraph::new(),
        }
    }

    /// Execute refresh operation with dependency resolution
    pub async fn refresh_with_deps(&self, resource: Resource, force: bool) -> Result<()> {
        if force {
            return self.execute_refresh(resource).await;
        }

        let plan = self.graph.execution_plan(resource)?;

        if plan.len() > 1 {
            log::info!("Auto-resolving dependencies: {:?}", plan);
        }

        for step in plan {
            log::info!("Executing: {:?} refresh", step);
            self.execute_refresh(step).await?;
        }

        Ok(())
    }

    async fn execute_refresh(&self, resource: Resource) -> Result<()> {
        match resource {
            Resource::Tournaments => {
                let mut service = IngestionService::new()?;
                service.run().await
            },
            Resource::Rankings => {
                let config = AppConfig::new();
                let service = ProcessingService::new(config)?;
                service.run()
            },
            Resource::Avatars => {
                // Call handler directly
                crate::cli::handlers::avatars::execute_avatar_refresh(None).await
            },
            Resource::Database => {
                anyhow::bail!("Database is not a refreshable resource")
            },
        }
    }
}
