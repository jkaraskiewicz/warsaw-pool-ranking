use chrono::NaiveDateTime;

// 3 years in days (3 * 365)
const HALF_LIFE_DAYS: f64 = 1095.0;

pub fn calculate_weight(game_date: NaiveDateTime, current_date: NaiveDateTime) -> f64 {
    let age_days = calculate_age_days(game_date, current_date);
    apply_exponential_decay(age_days)
}

fn calculate_age_days(game_date: NaiveDateTime, current_date: NaiveDateTime) -> i64 {
    let duration = current_date.signed_duration_since(game_date);
    duration.num_days().max(0)
}

fn apply_exponential_decay(age_days: i64) -> f64 {
    // formula: weight = exp(-λ × days_ago)
    // where λ = ln(2) / half_life_days
    let lambda = std::f64::consts::LN_2 / HALF_LIFE_DAYS;
    let decay_factor = -lambda * (age_days as f64);
    decay_factor.exp()
}

pub fn apply_weights_to_games(
    games: &mut [crate::domain::ExpandedGame],
    current_date: NaiveDateTime,
) {
    for game in games.iter_mut() {
        game.weight = calculate_weight(game.date, current_date);
    }
}
