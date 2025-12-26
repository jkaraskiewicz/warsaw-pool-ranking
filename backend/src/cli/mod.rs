pub mod dependencies;
pub mod prompts;
pub mod handlers;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "warsaw-pool-rating backend")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab-case")]
pub enum Command {
    /// Start the backend server
    Serve {
        #[arg(short, long, default_value_t = 8000)]
        port: u16,
    },

    /// Tournament data operations
    #[command(subcommand)]
    Tournaments(TournamentCommand),

    /// Rating calculation operations
    #[command(subcommand)]
    Rankings(RankingCommand),

    /// Player avatar operations
    #[command(subcommand)]
    Avatars(AvatarCommand),

    /// Database management
    #[command(subcommand)]
    Database(DatabaseCommand),
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab-case")]
pub enum TournamentCommand {
    /// Fetch tournament data from CueScore
    Refresh {
        /// Skip dependency checks and force refresh
        #[arg(long)]
        force: bool,
    },
    /// Delete all cached tournament data
    Prune {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// List cached tournaments
    List {
        /// Output format
        #[arg(short, long, default_value = "table")]
        format: OutputFormat,
        /// Maximum number to display
        #[arg(short, long)]
        limit: Option<usize>,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab-case")]
pub enum RankingCommand {
    /// Calculate player ratings from tournament data
    Refresh {
        /// Skip dependency checks
        #[arg(long)]
        force: bool,
    },
    /// Export ratings to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,
        /// Output format
        #[arg(short, long, default_value = "json")]
        format: ExportFormat,
    },
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab-case")]
pub enum AvatarCommand {
    /// Download/update player avatars
    Refresh {
        /// Specific player CueScore ID
        #[arg(long)]
        player_id: Option<i64>,
        /// Skip dependency checks
        #[arg(long)]
        force: bool,
    },
    /// Delete all avatar data
    Prune {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Show avatar storage statistics
    Stats,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab-case")]
pub enum DatabaseCommand {
    /// Reset database (drop all tables)
    Reset {
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
    /// Backup database to file
    Backup {
        /// Output file path
        #[arg(short, long)]
        to: String,
    },
    /// Restore database from backup
    Restore {
        /// Backup file path
        #[arg(short, long)]
        from: String,
        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum OutputFormat {
    Table,
    Json,
    Csv,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
}
