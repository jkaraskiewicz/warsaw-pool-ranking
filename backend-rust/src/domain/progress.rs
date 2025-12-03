use log::info;

/// Track progress of tournament fetching
pub struct FetchProgress {
    total: usize,
    fetched: usize,
    cached: usize,
}

impl FetchProgress {
    pub fn new(total: usize) -> Self {
        Self {
            total,
            fetched: 0,
            cached: 0,
        }
    }

    pub fn increment_fetched(&mut self) {
        self.fetched += 1;
        self.log_progress();
    }

    pub fn increment_cached(&mut self) {
        self.cached += 1;
        self.log_progress();
    }

    pub fn current_count(&self) -> usize {
        self.fetched + self.cached
    }

    fn log_progress(&self) {
        let current = self.current_count();
        if should_log(current, self.total) {
            info!(
                "  → Progress: {}/{} ({} new, {} cached)",
                current, self.total, self.fetched, self.cached
            );
        }
    }
}

fn should_log(current: usize, total: usize) -> bool {
    is_milestone(current) || is_complete(current, total)
}

fn is_milestone(count: usize) -> bool {
    count % 10 == 0
}

fn is_complete(current: usize, total: usize) -> bool {
    current == total
}
