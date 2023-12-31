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
        let meter_reading_1 = state.meterreadings[0].to.unwrap();
        let meter_reading_2 = state.meterreadings[1].to.unwrap();

        let tariff_indicator = state.tariff_indicator.unwrap();

        // stroom 1 = daluren
        // stroom 2 = piekuren

        let tariff: u8 = tariff_indicator[1];

        let gas_timestamp = utils::convert_tst(state.slaves[0].meter_reading.unwrap().0).unwrap();

        let gas = state.slaves[0].meter_reading.unwrap().1;

        let points = vec![
            DataPoint::builder("meter")
                .field("power_delivered", power_delivered)
                .field("voltage", voltage)
                .field("current", current)
                .field("active_power_plus", active_power_plus)
                .field("tariff", tariff as i64)
                .field("meter_reading_1", meter_reading_1)
                .field("meter_reading_2", meter_reading_2)
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
