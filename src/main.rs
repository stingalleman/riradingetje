mod prices;
mod utils;

use std::env;
use std::time::Duration;

use futures::prelude::*;
use influxdb2::models::DataPoint;
use influxdb2::Client;
use std::io::Read;
use tokio_cron_scheduler::{Job, JobScheduler};

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let tty_path = &args[1];
    let token = &args[2];

    // setup scheduler for energy prices fetching
    let sched = JobScheduler::new().await.unwrap();

    sched
        .add(
            Job::new_async("1/3 * * * * *", |_, _| {
                Box::pin(async {
                    let x = prices::get_prices().await.unwrap();
                    for a in x {
                        println!("{} @ {}", a.price, a.timestamp.to_rfc2822())
                    }
                })
            })
            .unwrap(),
        )
        .await
        .unwrap();

    // start scheduler
    sched.start().await.unwrap();

    // data bucket
    let bucket = "test2";
    let client = Client::new("https://influxdb.stingalleman.dev", "lab", token);

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
        println!("saved!");
    }
}
