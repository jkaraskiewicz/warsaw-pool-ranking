use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about = "warsaw-pool-rating backend")]
pub struct Cli {
    /// Command
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone, PartialEq)]
#[clap(rename_all = "lower_case")]
pub enum Command {
    /// Start the backend server
    Serve {
        /// Port number (optional, defaults to 3000)
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
    /// Fetch new data from CueScore and store it in cache and database
    Ingest,
    /// Calculate ratings based on data in the database
    Process,
}
