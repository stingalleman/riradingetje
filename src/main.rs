use std::env;
use std::time::Duration;

use chrono::TimeZone;
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
    let bucket = "test2";
    let client = Client::new("https://influxdb.stingalleman.dev", "lab", token);

    let port = serialport::new(tty_path, 115_200)
        .data_bits(serialport::DataBits::Eight)
        .stop_bits(serialport::StopBits::One)
        .parity(serialport::Parity::None)
        .timeout(Duration::from_millis(20000))
        .open()
        .expect("Failed to open port");

    let reader = dsmr5::Reader::new(port.bytes());

    for readout in reader {
        let x = readout.unwrap();
        let telegram = x.to_telegram().unwrap();
        let state = dsmr5::Result::<dsmr5::state::State>::from(&telegram).unwrap();

        let state_timestamp = state.datetime.unwrap();
        let year: i32 = state_timestamp.year.into();

        let timestamp = chrono::Local
            .with_ymd_and_hms(
                year + 2000,
                state_timestamp.month.into(),
                state_timestamp.day.into(),
                state_timestamp.hour.into(),
                state_timestamp.minute.into(),
                state_timestamp.second.into(),
            )
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap();

        let power_delivered = state.power_delivered.unwrap();
        let voltage = state.lines[0].voltage.unwrap();
        println!("{}", voltage);

        let points = vec![DataPoint::builder("meter")
            .field("power_delivered", power_delivered)
            .timestamp(timestamp)
            .build()
            .unwrap()];

        client.write(bucket, stream::iter(points)).await.unwrap();
    }
}
