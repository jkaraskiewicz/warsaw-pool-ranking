/// Venue configuration for tournament scraping
///
/// To find venue IDs on CueScore:
/// 1. Navigate to https://cuescore.com
/// 2. Search for the venue by name
/// 3. Click on the venue
/// 4. The URL will be: https://cuescore.com/venue/{name}/{id}/
/// 5. Extract the ID from the URL
#[derive(Debug, Clone)]
pub struct VenueConfig {
    pub id: i64,
    pub slug: &'static str,
    pub name: &'static str,
}

impl VenueConfig {
    pub fn new(id: i64, slug: &'static str, name: &'static str) -> Self {
        Self { id, slug, name }
    }
}

/// Get the list of Warsaw pool venues to scrape for tournaments
pub fn get_venues() -> Vec<VenueConfig> {
    vec![
        VenueConfig::new(2842336, "147 Break Zamieniecka", "147 Break Zamieniecka"),
        VenueConfig::new(29830615, "147 Break Fort Wola", "147 Break Fort Wola"),
        VenueConfig::new(1698108, "147 Break Nowogrodzka", "147 Break Nowogrodzka"),
        VenueConfig::new(57050170, "Shooters", "Shooters"),
        VenueConfig::new(3367445, "Eighty Nine", "Eighty Nine"),
        VenueConfig::new(36031138, "Złota Bila - Centrum Bilardowe", "Złota Bila - Centrum Bilardowe"),
        VenueConfig::new(2357769, "Billboard pool & snooker", "Billboard Pool & Snooker"),
        VenueConfig::new(1634568, "Klub Pictures", "Klub Pictures"),
        VenueConfig::new(22253992, "The Lounge - Billiards Club", "The Lounge - Billiards Club"),
    ]
}
