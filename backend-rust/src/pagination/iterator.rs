use super::config::PaginationConfig;

/// Iterator for paginated requests
pub struct PageIterator {
    current_page: usize,
    config: PaginationConfig,
}

impl PageIterator {
    pub fn new(config: PaginationConfig) -> Self {
        Self {
            current_page: 1,
            config,
        }
    }

    pub fn current_page(&self) -> usize {
        self.current_page
    }

    pub fn has_reached_max(&self) -> bool {
        self.config.max_pages.map_or(false, |max| self.current_page > max)
    }

    pub fn advance(&mut self) {
        self.current_page += 1;
    }

    pub fn is_first_page(&self) -> bool {
        self.current_page == 1
    }
}
