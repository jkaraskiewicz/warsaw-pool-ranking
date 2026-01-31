pub mod dependencies;
pub mod handlers;
pub mod prompts;

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

    /// Sync data from CueScore (fetch -> calculate -> avatars)
    Sync {
        /// Run specific step only: fetch, calculate, avatars
        #[arg(long)]
        step: Option<SyncStep>,

        /// Skip dependency checks
        #[arg(long)]
        force: bool,
    },

    /// Show system status (database, cache, avatars)
    Status,

    /// Export data to file
    Export {
        /// Output file path
        #[arg(short, long)]
        output: String,

        /// Data type to export
        #[arg(short, long, default_value = "ratings")]
        r#type: ExportType,

        /// Output format
        #[arg(long, default_value = "json")]
        format: ExportFormat,
    },

    /// Reset system (database + optionally cache)
    Reset {
        /// Keep tournament cache files
        #[arg(long)]
        keep_cache: bool,

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },

    /// Backup database
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

        /// Skip confirmation
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum SyncStep {
    Fetch,
    Calculate,
    Avatars,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum ExportType {
    Ratings,
    Tournaments,
}

#[derive(clap::ValueEnum, Debug, Clone, PartialEq)]
pub enum ExportFormat {
    Json,
    Csv,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Result<Command, clap::Error> {
        let cli = Cli::try_parse_from(args)?;
        Ok(cli.command)
    }

    #[test]
    fn test_serve_default_port() {
        let cmd = parse_args(&["app", "serve"]).unwrap();
        assert_eq!(cmd, Command::Serve { port: 8000 });
    }

    #[test]
    fn test_serve_custom_port() {
        let cmd = parse_args(&["app", "serve", "--port", "3000"]).unwrap();
        assert_eq!(cmd, Command::Serve { port: 3000 });
    }

    #[test]
    fn test_sync_full() {
        let cmd = parse_args(&["app", "sync"]).unwrap();
        assert_eq!(
            cmd,
            Command::Sync {
                step: None,
                force: false
            }
        );
    }

    #[test]
    fn test_sync_fetch_step() {
        let cmd = parse_args(&["app", "sync", "--step", "fetch"]).unwrap();
        assert_eq!(
            cmd,
            Command::Sync {
                step: Some(SyncStep::Fetch),
                force: false
            }
        );
    }

    #[test]
    fn test_sync_calculate_step() {
        let cmd = parse_args(&["app", "sync", "--step", "calculate"]).unwrap();
        assert_eq!(
            cmd,
            Command::Sync {
                step: Some(SyncStep::Calculate),
                force: false
            }
        );
    }

    #[test]
    fn test_sync_avatars_step() {
        let cmd = parse_args(&["app", "sync", "--step", "avatars"]).unwrap();
        assert_eq!(
            cmd,
            Command::Sync {
                step: Some(SyncStep::Avatars),
                force: false
            }
        );
    }

    #[test]
    fn test_sync_force() {
        let cmd = parse_args(&["app", "sync", "--force"]).unwrap();
        assert_eq!(
            cmd,
            Command::Sync {
                step: None,
                force: true
            }
        );
    }

    #[test]
    fn test_sync_step_with_force() {
        let cmd = parse_args(&["app", "sync", "--step", "fetch", "--force"]).unwrap();
        assert_eq!(
            cmd,
            Command::Sync {
                step: Some(SyncStep::Fetch),
                force: true
            }
        );
    }

    #[test]
    fn test_status() {
        let cmd = parse_args(&["app", "status"]).unwrap();
        assert_eq!(cmd, Command::Status);
    }

    #[test]
    fn test_export_ratings_json() {
        let cmd = parse_args(&["app", "export", "-o", "out.json"]).unwrap();
        assert_eq!(
            cmd,
            Command::Export {
                output: "out.json".to_string(),
                r#type: ExportType::Ratings,
                format: ExportFormat::Json,
            }
        );
    }

    #[test]
    fn test_export_tournaments_csv() {
        let cmd =
            parse_args(&["app", "export", "-o", "t.csv", "--type", "tournaments", "--format", "csv"])
                .unwrap();
        assert_eq!(
            cmd,
            Command::Export {
                output: "t.csv".to_string(),
                r#type: ExportType::Tournaments,
                format: ExportFormat::Csv,
            }
        );
    }

    #[test]
    fn test_reset_defaults() {
        let cmd = parse_args(&["app", "reset"]).unwrap();
        assert_eq!(
            cmd,
            Command::Reset {
                keep_cache: false,
                yes: false
            }
        );
    }

    #[test]
    fn test_reset_keep_cache() {
        let cmd = parse_args(&["app", "reset", "--keep-cache"]).unwrap();
        assert_eq!(
            cmd,
            Command::Reset {
                keep_cache: true,
                yes: false
            }
        );
    }

    #[test]
    fn test_reset_yes() {
        let cmd = parse_args(&["app", "reset", "-y"]).unwrap();
        assert_eq!(
            cmd,
            Command::Reset {
                keep_cache: false,
                yes: true
            }
        );
    }

    #[test]
    fn test_backup() {
        let cmd = parse_args(&["app", "backup", "--to", "backup.db"]).unwrap();
        assert_eq!(
            cmd,
            Command::Backup {
                to: "backup.db".to_string()
            }
        );
    }

    #[test]
    fn test_restore() {
        let cmd = parse_args(&["app", "restore", "--from", "backup.db"]).unwrap();
        assert_eq!(
            cmd,
            Command::Restore {
                from: "backup.db".to_string(),
                yes: false
            }
        );
    }

    #[test]
    fn test_restore_yes() {
        let cmd = parse_args(&["app", "restore", "--from", "backup.db", "-y"]).unwrap();
        assert_eq!(
            cmd,
            Command::Restore {
                from: "backup.db".to_string(),
                yes: true
            }
        );
    }

    #[test]
    fn test_invalid_sync_step() {
        let result = parse_args(&["app", "sync", "--step", "invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_export_missing_output() {
        let result = parse_args(&["app", "export"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_backup_missing_to() {
        let result = parse_args(&["app", "backup"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_restore_missing_from() {
        let result = parse_args(&["app", "restore"]);
        assert!(result.is_err());
    }
}
