mod prices;
mod utils;

use std::time::Duration;

use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use std::io::Read;
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::InfluxConfig;

#[tokio::main]
async fn main() {
    let tty_path = std::env::args().nth(1).expect("no tty_path given");
    let token = std::env::args().nth(2).expect("no token given");

    let influx_config = InfluxConfig {
        bucket: "slimme-meter".to_string(),
        org: "slimme-meter".to_string(),
        token,
        url: "https://influxdb.stingalleman.dev".to_string(),
    }
    .clone();

    let binding = influx_config.bucket.clone();
    let bucket = binding.as_str();

    let client = Client::new(
        influx_config.url.clone(),
        influx_config.org.clone(),
        influx_config.token.clone(),
    );

    // setup scheduler for energy prices fetching
    let sched = JobScheduler::new().await.unwrap();

    sched
        .add(
            Job::new_async("0 * * * * *", move |_, _| {
                let influx_config = influx_config.clone();

                Box::pin(async move {
                    prices::publish_prices(influx_config).await;
                })
            })
            .unwrap(),
        )
        .await
        .unwrap();

    // start scheduler
    sched.start().await.unwrap();

    // setup serial port
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

        let timestamp = utils::convert_tst(state.datetime.unwrap()).unwrap();

        let power_delivered = state.power_delivered.unwrap();
        let voltage = state.lines[0].voltage.unwrap();
        let current: i64 = state.lines[0].current.unwrap() as i64;
        let active_power_plus = state.lines[0].active_power_plus.unwrap();
        let a = state.meterreadings[0].to.unwrap();
        let b = state.meterreadings[1].to.unwrap();

        println!("{}, {}, {}", a, b, "a");

        let tariff_indicator = state.tariff_indicator.unwrap();
        let tariff: u8;
        if tariff_indicator[0] == 1 {
            tariff = 1;
        } else if tariff_indicator[1] == 1 {
            tariff = 2;
        } else {
            tariff = 3;
        }

        let gas_timestamp = utils::convert_tst(state.slaves[0].meter_reading.unwrap().0).unwrap();

        let gas = state.slaves[0].meter_reading.unwrap().1;

        let points = vec![
            DataPoint::builder("meter")
                .field("power_delivered", power_delivered)
                .field("voltage", voltage)
                .field("current", current)
                .field("active_power_plus", active_power_plus)
                .field("tariff", tariff as i64)
                .field("x", a)
                .timestamp(timestamp)
                .build()
                .unwrap(),
            DataPoint::builder("gas-meter")
                .field("gas", gas)
                .timestamp(gas_timestamp)
                .build()
                .unwrap(),
        ];

        client.write(bucket, stream::iter(points)).await.unwrap();
    }
}
