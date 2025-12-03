use crate::domain::models::TournamentResponse;
use anyhow::Result;
use serde_json::Value;

/// Parse tournament from raw API response
pub fn parse_tournament(raw: &Value) -> Result<TournamentResponse> {
    let tournament: TournamentResponse = serde_json::from_value(raw.clone())?;
    Ok(tournament)
}

/// Check if tournament has matches
pub fn has_matches(tournament: &TournamentResponse) -> bool {
    !tournament.matches.is_empty()
}

/// Get match count
pub fn match_count(tournament: &TournamentResponse) -> usize {
    tournament.matches.len()
}
