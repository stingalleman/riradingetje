use std::env;

use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;

#[tokio::main]
async fn main() {
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| "/dev/ttyUSB0".into());
    let token = args.nth(2).unwrap();

    let bucket = "test";
    let client = Client::new("https://influxdb.stingalleman.dev", "lab", token);

    let points = vec![DataPoint::builder("cpu")
        .tag("host", "server01")
        .field("usage", 0.5)
        .build()
        .unwrap()];

    client.write(bucket, stream::iter(points)).await.unwrap();

    // let port = serialport::new(tty_path, 115_200)
    //     .data_bits(serialport::DataBits::Eight)
    //     .stop_bits(serialport::StopBits::One)
    //     .parity(serialport::Parity::None)
    //     .timeout(Duration::from_millis(2000))
    //     .open()
    //     .expect("Failed to open port");

    // let reader = dsmr5::Reader::new(port.bytes());

    // for readout in reader {
    //     let x = readout.unwrap();
    //     let telegram = x.to_telegram().unwrap();
    //     let state = dsmr5::Result::<dsmr5::state::State>::from(&telegram).unwrap();
    // }
}
