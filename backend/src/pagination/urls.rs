/// Build paginated URL with ?page= parameter
pub fn build_paginated_url(base_url: &str, page: usize) -> String {
    if is_first_page(page) {
        base_url.to_string()
    } else {
        format_with_page_param(base_url, page)
    }
}

/// Build paginated URL with &page= or ?page= based on existing params
pub fn build_paginated_url_with_params(base_url: &str, page: usize) -> String {
    if is_first_page(page) {
        base_url.to_string()
    } else {
        let separator = determine_separator(base_url);
        format!("{}{}page={}", base_url, separator, page)
    }
}

fn is_first_page(page: usize) -> bool {
    page == 1
}

fn format_with_page_param(base: &str, page: usize) -> String {
    format!("{}?page={}", base, page)
}

fn determine_separator(url: &str) -> char {
    if url.contains('?') { '&' } else { '?' }
}
