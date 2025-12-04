
use anyhow::Result;
use reqwest::Client;
use tokio::time::{sleep, Duration};

const BASE_URL: &str = "https://cuescore.com";
const USER_AGENT: &str = "WarsawPoolRankings/2.0";
const VENUE_ID: i64 = 1698108; // 147-break-nowogrodzka
const VENUE_NAME: &str = "147-break-nowogrodzka";

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()?;

    let venue_name_encoded = urlencoding::encode(VENUE_NAME).replace("%20", "+");
    let url = format!("{}/venue/{}/{}/tournaments", BASE_URL, venue_name_encoded, VENUE_ID);

    println!("Attempting to fetch URL: {}", url);

    sleep(Duration::from_millis(1000)).await; // Respect rate limit

    let response = client.get(&url).send().await?;

    println!("Response Status: {}", response.status());

    if response.status().is_success() {
        let text = response.text().await?;
        println!("Response Body (first 500 chars):
{}", &text[0..std::cmp::min(text.len(), 500)]);
    } else {
        println!("Failed to get successful response.");
    }

    Ok(())
}
