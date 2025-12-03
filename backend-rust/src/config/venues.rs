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
    pub name: &'static str,
}

impl VenueConfig {
    pub fn new(id: i64, name: &'static str) -> Self {
        Self { id, name }
    }
}

/// Get the list of Warsaw pool venues to scrape for tournaments
pub fn get_venues() -> Vec<VenueConfig> {
    vec![
        VenueConfig::new(2842336, "147-break-zamieniecka"),
        VenueConfig::new(2983061, "147-break-fort-wola"),
        VenueConfig::new(1698108, "147-break-nowogrodzka"),
        VenueConfig::new(57050170, "shooters"),
        VenueConfig::new(3367445, "eighty-nine"),
        VenueConfig::new(36031138, "zlota-bila-centrum-bilardowe"),
        VenueConfig::new(2357769, "billboard-pool-snooker"),
        VenueConfig::new(1634568, "klub-pictures"),
        VenueConfig::new(22253992, "the-lounge-billiards-club"),
    ]
}
