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
        VenueConfig::new(2842336, "147-break-zamieniecka", "147 Break Zamieniecka"),
        VenueConfig::new(29830615, "147-break-fort-wola", "147 Break Fort Wola"),
        VenueConfig::new(1698108, "147-break-nowogrodzka", "147 Break Nowogrodzka"),
        VenueConfig::new(57050170, "shooters", "Shooters"),
        VenueConfig::new(3367445, "eighty-nine", "Eighty Nine"),
        VenueConfig::new(36031138, "zlota-bila-centrum-bilardowe", "Złota Bila - Centrum Bilardowe"),
        VenueConfig::new(2357769, "billboard-pool-snooker", "Billboard Pool & Snooker"),
        VenueConfig::new(1634568, "klub-pictures", "Klub Pictures"),
        VenueConfig::new(22253992, "the-lounge-billiards-club", "The Lounge - Billiards Club"),
    ]
}
