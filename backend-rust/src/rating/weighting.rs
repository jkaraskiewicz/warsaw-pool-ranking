use chrono::NaiveDateTime;

const DECAY_DAYS: i64 = 90;
const MIN_WEIGHT: f64 = 0.5;

pub fn calculate_weight(game_date: NaiveDateTime, current_date: NaiveDateTime) -> f64 {
    let age_days = calculate_age_days(game_date, current_date);
    apply_exponential_decay(age_days)
}

fn calculate_age_days(game_date: NaiveDateTime, current_date: NaiveDateTime) -> i64 {
    let duration = current_date.signed_duration_since(game_date);
    duration.num_days().max(0)
}

fn apply_exponential_decay(age_days: i64) -> f64 {
    let decay_factor = -(age_days as f64) / (DECAY_DAYS as f64);
    let weight = decay_factor.exp();
    weight.max(MIN_WEIGHT)
}

pub fn apply_weights_to_games(
    games: &mut [crate::domain::ExpandedGame],
    current_date: NaiveDateTime,
) {
    for game in games.iter_mut() {
        game.weight = calculate_weight(game.date, current_date);
    }
}
