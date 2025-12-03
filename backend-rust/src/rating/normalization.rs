use super::types::RatingMap;

const TARGET_MEAN: f64 = 1500.0;
const TARGET_STD_DEV: f64 = 200.0;

pub fn normalize_ratings(ratings: &mut RatingMap) {
    let mean = calculate_mean(ratings);
    let std_dev = calculate_std_dev(ratings, mean);

    if std_dev > 0.0 {
        apply_normalization(ratings, mean, std_dev);
    }
}

fn calculate_mean(ratings: &RatingMap) -> f64 {
    let sum: f64 = ratings.values().sum();
    sum / ratings.len() as f64
}

fn calculate_std_dev(ratings: &RatingMap, mean: f64) -> f64 {
    let variance = calculate_variance(ratings, mean);
    variance.sqrt()
}

fn calculate_variance(ratings: &RatingMap, mean: f64) -> f64 {
    let sum_sq_diff: f64 = ratings
        .values()
        .map(|&r| (r - mean).powi(2))
        .sum();

    sum_sq_diff / ratings.len() as f64
}

fn apply_normalization(
    ratings: &mut RatingMap,
    mean: f64,
    std_dev: f64,
) {
    for rating in ratings.values_mut() {
        *rating = transform_rating(*rating, mean, std_dev);
    }
}

fn transform_rating(rating: f64, mean: f64, std_dev: f64) -> f64 {
    let z_score = (rating - mean) / std_dev;
    TARGET_MEAN + (z_score * TARGET_STD_DEV)
}
