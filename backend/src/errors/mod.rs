use anyhow::Context as _;

/// Add context to fetch errors
pub fn fetch_context(url: &str) -> String {
    format!("Failed to fetch from: {}", url)
}

/// Add context to parse errors
pub fn parse_context(data_type: &str) -> String {
    format!("Failed to parse {}", data_type)
}

/// Add context to cache errors
pub fn cache_context(operation: &str, key: &str) -> String {
    format!("Failed to {} cache for key: {}", operation, key)
}

/// Wrap result with fetch context
pub fn with_fetch_context<T, E>(result: Result<T, E>, url: &str) -> anyhow::Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
{
    result.context(fetch_context(url))
}

/// Wrap result with parse context
pub fn with_parse_context<T, E>(result: Result<T, E>, data_type: &str) -> anyhow::Result<T>
where
    E: std::error::Error + Send + Sync + 'static,
{
    result.context(parse_context(data_type))
}
