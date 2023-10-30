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
    let resp = reqwest::get("https://api.energyzero.nl/v1/energyprices?fromDate=2023-10-28T22%3A00%3A00.000Z&tillDate=2023-10-29T22%3A59%3A59.999Z&interval=4&usageType=1&inclBtw=true")
        .await?
        .json::<EnergyZeroApi>()
        .await?;

    for x in &resp.prices {
        let chrono_timestamp = chrono::DateTime::parse_from_rfc3339(&x.reading_date).unwrap();
        println!("{}", chrono_timestamp.timestamp());
    }

    Ok(resp)
}
