use serde_json::Value;

/// Check if API response has more pages
pub fn has_more_pages(data: &Value) -> bool {
    extract_has_more(data).unwrap_or(false)
}

/// Extract pagination info from response
fn extract_has_more(data: &Value) -> Option<bool> {
    data.get("pagination")?.get("has_more")?.as_bool()
}

/// Check if response indicates end of data
pub fn is_empty_response(data: &Value) -> bool {
    extract_items(data).map_or(true, |items| items.is_empty())
}

fn extract_items(data: &Value) -> Option<&Vec<Value>> {
    data.get("data")?.as_array()
}
