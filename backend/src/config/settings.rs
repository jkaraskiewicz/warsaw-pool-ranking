#[derive(Debug, Clone)]
pub struct RatingPeriod {
    pub name: String,
    pub years: Option<u32>, // None implies "All Time"
}

#[derive(Debug, Clone)]
pub struct RatingSettings {
    pub starter_rating: f64,
    pub virtual_games_weight: f64,
    pub min_ranked_games: i32,
    pub established_games: i32,
    pub convergence_tolerance: f64,
    pub max_iterations: usize,
    pub periods: Vec<RatingPeriod>,
}

impl Default for RatingSettings {
    fn default() -> Self {
        Self {
            starter_rating: 500.0,
            virtual_games_weight: 5.0,
            min_ranked_games: 50,
            established_games: 200,
            convergence_tolerance: 1e-6,
            max_iterations: 100,
            periods: vec![
                RatingPeriod { name: "all".to_string(), years: None },
                RatingPeriod { name: "1y".to_string(), years: Some(1) },
                RatingPeriod { name: "2y".to_string(), years: Some(2) },
                RatingPeriod { name: "3y".to_string(), years: Some(3) },
                RatingPeriod { name: "4y".to_string(), years: Some(4) },
                RatingPeriod { name: "5y".to_string(), years: Some(5) },
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScraperSettings {
    pub rate_limit_ms: u64,
    pub user_agent: &'static str,
    pub timeout_secs: u64,
    pub base_url: &'static str,
    pub api_base_url: &'static str,
}

impl Default for ScraperSettings {
    fn default() -> Self {
        Self {
            rate_limit_ms: 100, // 10 req/sec
            user_agent: "WarsawPoolRankings/2.0",
            timeout_secs: 30,
            base_url: "https://cuescore.com",
            api_base_url: "https://api.cuescore.com",
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdminSettings {
    pub password: String,
}

impl Default for AdminSettings {
    fn default() -> Self {
        Self::new()
    }
}

impl AdminSettings {
    pub fn new() -> Self {
        let password = std::env::var("ADMIN_PASSWORD")
            .unwrap_or_else(|_| {
                log::warn!("ADMIN_PASSWORD not set, using default 'admin' (INSECURE for production!)");
                "admin".to_string()
            });
        Self { password }
    }
}

#[derive(Debug, Clone)]
pub struct AvatarSize {
    pub name: &'static str,
    pub pixels: u32,
}

#[derive(Debug, Clone)]
pub struct AvatarSettings {
    pub sizes: Vec<AvatarSize>,
    pub max_width: u32,
    pub max_height: u32,
    pub quality: u32,
}

impl Default for AvatarSettings {
    fn default() -> Self {
        Self {
            sizes: vec![
                AvatarSize { name: "small", pixels: 60 },
                AvatarSize { name: "medium", pixels: 120 },
                AvatarSize { name: "large", pixels: 144 },
            ],
            max_width: 1000,
            max_height: 4000,
            quality: 80,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TournamentProcessingSettings {
    pub doubles_keywords: Vec<&'static str>,
    pub team_player_separators: Vec<&'static str>,
    pub team_player_prefixes: Vec<&'static str>,
}

impl Default for TournamentProcessingSettings {
    fn default() -> Self {
        Self {
            doubles_keywords: vec!["debel", "deblowy", "double", " par", "pary", "team"],
            team_player_separators: vec!["/", "&", "+"],
            team_player_prefixes: vec!["team", "6ur"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub rating: RatingSettings,
    pub scraper: ScraperSettings,
    pub admin: AdminSettings,
    pub avatar: AvatarSettings,
    pub tournament_processing: TournamentProcessingSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            rating: RatingSettings::default(),
            scraper: ScraperSettings::default(),
            admin: AdminSettings::new(),
            avatar: AvatarSettings::default(),
            tournament_processing: TournamentProcessingSettings::default(),
        }
    }
}

// Lazy static or just regular instantiation?
// Since we are refactoring for "small methods/classes", we should prefer
// passing the config explicitly (Dependency Injection) rather than globals.
