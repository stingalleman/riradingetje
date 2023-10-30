use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct EnergyZeroApi {
    #[serde(rename = "Prices")]
    prices: Vec<Price>,

    #[serde(rename = "intervalType")]
    interval_type: i64,

    #[serde(rename = "average")]
    average: f64,

    #[serde(rename = "fromDate")]
    from_date: String,

    #[serde(rename = "tillDate")]
    till_date: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Price {
    #[serde(rename = "price")]
    price: f64,

    #[serde(rename = "readingDate")]
    reading_date: String,
}

pub struct Prices {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
}

pub async fn get_prices() -> Result<Vec<Prices>, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();

    let from_date = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let till_date = (now + Duration::days(1))
        .date_naive()
        .and_hms_opt(23, 59, 59)
        .unwrap()
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();

    let url = format!("https://api.energyzero.nl/v1/energyprices?fromDate={}&tillDate={}&interval=4&usageType=1&inclBtw=true", from_date, till_date);

    let resp = reqwest::get(url).await?.json::<EnergyZeroApi>().await?;

    let mut buf: Vec<Prices> = vec![];

    for item in &resp.prices {
        let timestamp = chrono::DateTime::parse_from_rfc3339(&item.reading_date).unwrap();

        buf.push(Prices {
            timestamp: timestamp.into(),
            price: item.price,
        });
    }

    Ok(buf)
}
