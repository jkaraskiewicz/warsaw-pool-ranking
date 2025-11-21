"""Venue configuration for Warsaw pool venues.

To add a venue, you need:
1. venue_id - from the CueScore URL
2. slug - the URL-friendly name from the CueScore URL
3. name - display name

Example venue URL: https://cuescore.com/venue/147-break-nowogrodzka/67496954
- venue_id: "67496954"
- slug: "147-break-nowogrodzka"
- name: "147 Break Nowogrodzka"

Note: The base venue URL is used. The scraper automatically appends /tournaments
when fetching tournament lists.
"""

# Warsaw venues to track
WARSAW_VENUES = [
    {
        "id": "2842336",
        "slug": "147-break-zamieniecka",
        "name": "147 Break Zamieniecka"
    },
    {
        "id": "2983061",
        "slug": "147-break-fort-wola",
        "name": "147 Break Fort Wola"
    },
    {
        "id": "1698108",
        "slug": "147-break-nowogrodzka",
        "name": "147 Break Nowogrodzka"
    },
    {
        "id": "57050170",
        "slug": "shooters",
        "name": "Shooters"
    },
    {
        "id": "3367445",
        "slug": "eighty-nine",
        "name": "Eighty Nine"
    },
    {
        "id": "36031138",
        "slug": "zlota-bila-centrum-bilardowe",
        "name": "Złota Bila - Centrum Bilardowe"
    },
    {
        "id": "2357769",
        "slug": "billboard-pool-snooker",
        "name": "Billboard Pool & Snooker"
    },
    {
        "id": "1634568",
        "slug": "klub-pictures",
        "name": "Klub Pictures"
    },
    {
        "id": "22253992",
        "slug": "the-lounge-billiards-club",
        "name": "The Lounge - Billiards Club"
    },
]
