pub fn calculate_adjusted_ratings(
    games_played: i32,
    rating: f64,
    established_games: i32,
    starter_rating: f64,
) -> (f64, f64, f64) {
    let starter_weight = if games_played >= established_games {
        0.0
    } else {
        (established_games - games_played) as f64 / established_games as f64
    };
    let ml_weight = 1.0 - starter_weight;

    let ml_rating = if ml_weight > 0.0001 {
        (rating - (starter_weight * starter_rating)) / ml_weight
    } else {
        rating
    };
    (ml_rating, starter_weight, ml_weight)
}
