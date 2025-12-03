use super::types::RatingMap;

const CONVERGENCE_THRESHOLD: f64 = 0.0001;
const MAX_ITERATIONS: usize = 1000;

pub fn has_converged(
    old_ratings: &RatingMap,
    new_ratings: &RatingMap,
) -> bool {
    let max_change = calculate_max_change(old_ratings, new_ratings);
    max_change < CONVERGENCE_THRESHOLD
}

fn calculate_max_change(
    old_ratings: &RatingMap,
    new_ratings: &RatingMap,
) -> f64 {
    old_ratings
        .iter()
        .map(|(id, &old_val)| compute_change(old_val, new_ratings, id))
        .fold(0.0, f64::max)
}

fn compute_change(
    old_val: f64,
    new_ratings: &RatingMap,
    id: &i32,
) -> f64 {
    let new_val = new_ratings.get(id).copied().unwrap_or(old_val);
    (new_val - old_val).abs()
}

pub fn should_continue(iteration: usize) -> bool {
    iteration < MAX_ITERATIONS
}
