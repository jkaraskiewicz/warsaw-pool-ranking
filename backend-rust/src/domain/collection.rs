use super::models::TournamentResponse;
use std::collections::HashMap;

/// Collection of tournaments indexed by ID
pub struct TournamentCollection {
    tournaments: HashMap<i64, TournamentResponse>,
}

impl TournamentCollection {
    pub fn new() -> Self {
        Self {
            tournaments: HashMap::new(),
        }
    }

    pub fn add(&mut self, tournament: TournamentResponse) {
        self.tournaments.insert(tournament.id, tournament);
    }

    pub fn len(&self) -> usize {
        self.tournaments.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tournaments.is_empty()
    }

    pub fn get(&self, id: i64) -> Option<&TournamentResponse> {
        self.tournaments.get(&id)
    }

    pub fn into_vec(self) -> Vec<TournamentResponse> {
        self.tournaments.into_values().collect()
    }
}

impl Default for TournamentCollection {
    fn default() -> Self {
        Self::new()
    }
}
