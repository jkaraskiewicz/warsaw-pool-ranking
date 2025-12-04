use serde_json::Value;

/// Check if API response has more pages
pub fn has_more_pages(data: &Value) -> bool {
    extract_has_more(data).unwrap_or(false)
}

/// Extract pagination info from response
fn extract_has_more(data: &Value) -> Option<bool> {
    data.get("pagination")?.get("has_more")?.as_bool()
}
