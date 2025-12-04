/// Configuration for paginated requests
#[derive(Debug, Clone)]
pub struct PaginationConfig {
    pub max_pages: Option<usize>,
}

impl PaginationConfig {
    pub fn new() -> Self {
        Self { max_pages: None }
    }

    pub fn with_max_pages(mut self, max: usize) -> Self {
        self.max_pages = Some(max);
        self
    }
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self::new()
    }
}
