use std::os::unix::fs::chroot;

use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct EnergyZeroApi {
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
pub struct Price {
    #[serde(rename = "price")]
    price: f64,

    #[serde(rename = "readingDate")]
    reading_date: String,
}

pub async fn get_prices() -> Result<EnergyZeroApi, Box<dyn std::error::Error>> {
    let now = chrono::Utc::now();

    let today = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let tomorrow = (now + Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap();

    println!("{:?} - {:?}", today, tomorrow);

    let from_date = "2023-10-28T22:00:00.000Z";

    let till_date = "2023-10-29T22:59:59.999Z";

    let resp = reqwest::get(format!("https://api.energyzero.nl/v1/energyprices?fromDate={}&tillDate={}&interval=4&usageType=1&inclBtw=true", from_date, till_date))
        .await?
        .json::<EnergyZeroApi>()
        .await?;

    for x in &resp.prices {
        let chrono_timestamp = chrono::DateTime::parse_from_rfc3339(&x.reading_date).unwrap();
        // println!(
        //     "{} @ {} - {}",
        //     x.price,
        //     x.reading_date,
        //     chrono_timestamp.timestamp()
        // );
    }

    Ok(resp)
}
