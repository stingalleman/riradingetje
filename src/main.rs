use std::env;
use std::time::Duration;

use chrono::TimeZone;
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

    let sched = JobScheduler::new().await.unwrap();

    sched
        .add(
            Job::new("1/10 * * * * *", |uuid, l| {
                println!("I run every 10 seconds");
            })
            .unwrap(),
        )
        .await
        .unwrap();
    sched.start().await.unwrap();

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

        let gas_state_timestamp = state.slaves[0].meter_reading.unwrap().0;

        let gas_timestamp = chrono::Local
            .with_ymd_and_hms(
                year + 2000,
                gas_state_timestamp.month.into(),
                gas_state_timestamp.day.into(),
                gas_state_timestamp.hour.into(),
                gas_state_timestamp.minute.into(),
                gas_state_timestamp.second.into(),
            )
            .unwrap()
            .timestamp_nanos_opt()
            .unwrap();

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
