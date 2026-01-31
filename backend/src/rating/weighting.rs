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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_date(year: i32, month: u32, day: u32) -> NaiveDateTime {
        chrono::NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
    }

    #[test]
    fn test_weight_today_is_one() {
        let now = make_date(2024, 6, 15);
        let weight = calculate_weight(now, now);
        assert!((weight - 1.0).abs() < 1e-10, "Weight for same day should be 1.0");
    }

    #[test]
    fn test_weight_half_life_is_half() {
        let current = make_date(2024, 6, 15);
        let three_years_ago = make_date(2021, 6, 15);
        let weight = calculate_weight(three_years_ago, current);
        // After 3 years (half-life), weight should be ~0.5
        assert!(
            (weight - 0.5).abs() < 0.01,
            "Weight after half-life should be ~0.5, got {}",
            weight
        );
    }

    #[test]
    fn test_weight_double_half_life_is_quarter() {
        let current = make_date(2024, 6, 15);
        let six_years_ago = make_date(2018, 6, 15);
        let weight = calculate_weight(six_years_ago, current);
        // After 6 years (2 half-lives), weight should be ~0.25
        assert!(
            (weight - 0.25).abs() < 0.01,
            "Weight after 2 half-lives should be ~0.25, got {}",
            weight
        );
    }

    #[test]
    fn test_weight_decreases_monotonically() {
        let current = make_date(2024, 6, 15);
        let one_year_ago = make_date(2023, 6, 15);
        let two_years_ago = make_date(2022, 6, 15);
        let three_years_ago = make_date(2021, 6, 15);

        let w1 = calculate_weight(one_year_ago, current);
        let w2 = calculate_weight(two_years_ago, current);
        let w3 = calculate_weight(three_years_ago, current);

        assert!(w1 > w2, "1 year should weigh more than 2 years");
        assert!(w2 > w3, "2 years should weigh more than 3 years");
    }

    #[test]
    fn test_weight_never_negative() {
        let current = make_date(2024, 6, 15);
        let ancient = make_date(2000, 1, 1);
        let weight = calculate_weight(ancient, current);
        assert!(weight > 0.0, "Weight should never be negative");
    }

    #[test]
    fn test_future_date_treated_as_zero_age() {
        let current = make_date(2024, 6, 15);
        let future = make_date(2025, 1, 1);
        let weight = calculate_weight(future, current);
        // Future dates should have max(0, age), so weight = 1.0
        assert!((weight - 1.0).abs() < 1e-10, "Future date should have weight 1.0");
    }
}
