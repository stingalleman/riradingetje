use chrono::{DateTime, Utc};
use influxdb::InfluxDbWriteable;
use influxdb::{Client, Query, ReadQuery, Timestamp};
use std::{env, io::Read, time::Duration};

#[tokio::main]
async fn main() {
    let client = Client::new("https://influxdb.stingalleman.dev", "test");

    #[derive(InfluxDbWriteable)]
    struct WeatherReading {
        time: DateTime<Utc>,
        humidity: i32,
        #[influxdb(tag)]
        wind_direction: String,
    }

    // Let's write some data into a measurement called `weather`
    let weather_readings = vec![
        WeatherReading {
            time: Timestamp::Hours(1).into(),
            humidity: 30,
            wind_direction: String::from("north"),
        }
        .into_query("weather"),
        WeatherReading {
            time: Timestamp::Hours(2).into(),
            humidity: 40,
            wind_direction: String::from("west"),
        }
        .into_query("weather"),
    ];

    let write_result = client.query(weather_readings).await;
    assert!(write_result.is_ok(), "Write result was not okay");

    // Let's see if the data we wrote is there
    let read_query = ReadQuery::new("SELECT * FROM weather");

    let read_result = client.query(read_query).await;
    assert!(read_result.is_ok(), "Read result was not ok");
    println!("{}", read_result.unwrap());

    // let mut args = env::args();
    // let tty_path = args.nth(1).unwrap_or_else(|| "/dev/ttyUSB0".into());

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
