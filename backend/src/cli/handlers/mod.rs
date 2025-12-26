pub mod serve;
pub mod tournaments;
pub mod rankings;
pub mod avatars;
pub mod database;

pub use serve::handle_serve;
pub use tournaments::handle_tournament_command;
pub use rankings::handle_ranking_command;
pub use avatars::handle_avatar_command;
pub use database::handle_database_command;
