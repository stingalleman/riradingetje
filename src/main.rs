use std::env;
use std::time::Duration;

use chrono::NaiveDate;
use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use std::io::Read;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let tty_path = &args[1];
    let token = &args[2];

    println!("{} - {}", tty_path, token);
    let bucket = "test-meter";
    let client = Client::new("https://influxdb.stingalleman.dev", "lab", token);

    let port = serialport::new(tty_path, 115_200)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(2000))
        .open()
        .expect("Failed to open port");

    let reader = dsmr5::Reader::new(port.bytes());

    for readout in reader {
        let x = readout.unwrap();
        let telegram = x.to_telegram().unwrap();
        let state = dsmr5::Result::<dsmr5::state::State>::from(&telegram).unwrap();

        let state_timestamp = state.datetime.unwrap();
        let timestamp = NaiveDate::from_ymd_opt(
            state_timestamp.year.into(),
            state_timestamp.month.into(),
            state_timestamp.day.into(),
        )
        .unwrap()
        .and_hms_opt(
            state_timestamp.hour.into(),
            state_timestamp.minute.into(),
            state_timestamp.second.into(),
        )
        .unwrap()
        .timestamp();

        println!("{}", timestamp);

        let points: Vec<DataPoint> = vec![DataPoint::builder("meter")
            // .tag("host", "server01")
            .field("power_delivered", state.power_delivered.unwrap())
            .timestamp(timestamp)
            .build()
            .unwrap()];

        client.write(bucket, stream::iter(points)).await.unwrap();
    }
}
