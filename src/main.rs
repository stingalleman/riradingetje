use futures::stream::StreamExt;
use std::{env, io, str};
use tokio_util::codec::Decoder;

use bytes::BytesMut;
use tokio_serial::SerialPortBuilderExt;

struct DSMR {
    version: u8,
    // YYMMDDhhmm, ssX
    timestamp: u64,
    delivered_1: f64,
    delivered_2: f64,
    tarief: u8,
    delivered: f64,
    voltage: f32,
    current: f32,
    instantaneous_power: f64,
}
struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let newline = src.as_ref().iter().position(|b| *b == b'\n');
        if let Some(n) = newline {
            let line = src.split_to(n + 1);
            return match str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
            };
        }
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| "/dev/ttyUSB0".into());

    let mut port = tokio_serial::new(tty_path, 115200)
        .data_bits(tokio_serial::DataBits::Eight)
        .stop_bits(tokio_serial::StopBits::One)
        .parity(tokio_serial::Parity::None)
        .open_native_async()?;

    #[cfg(unix)]
    port.set_exclusive(false)
        .expect("Unable to set serial port exclusive to false");

    let mut reader = LineCodec.framed(port);

    let mut buf: DSMR = DSMR {
        version: 0,
        timestamp: 0,
        delivered_1: 0.0,
        delivered_2: 0.0,
        tarief: 0,
        delivered: 0.0,
        voltage: 0.0,
        current: 0.0,
        instantaneous_power: 0.0,
    };

    while let Some(line_result) = reader.next().await {
        let line = line_result.expect("Failed to read line");

        if line.contains("/") {
            buf = DSMR {
                version: 0,
                timestamp: 0,
                delivered_1: 0.0,
                delivered_2: 0.0,
                tarief: 0,
                delivered: 0.0,
                voltage: 0.0,
                current: 0.0,
                instantaneous_power: 0.0,
            };
        } else if line.contains("!") {
            println!("finished");
        }

        if line.contains("0-0:1.0.0") {
            let mut start = line.find("(").unwrap_or(0);
            if start != 0 {
                start = start + 1;
            }
            let end = line.find(")").unwrap_or(line.len());
            let res = &line[start..end];
            let x = res.to_string().strip_suffix("S").unwrap().to_string();

            buf.timestamp = x.parse().unwrap();
        }
        // let mut start = line.find("(").unwrap_or(0);
        // if start != 0 {
        //     start = start + 1;
        // }
        // let end = line.find(")").unwrap_or(line.len());
        // let res = &line[start..end];

        // println!("{}", res)

        //         let start_bytes = line.find("pattern").unwrap_or(0); //index where "pattern" starts
        //                                                      // or beginning of line if
        //                                                      // "pattern" not found
        // let end_bytes = line.find("<").unwrap_or(line.len()); //index where "<" is found
        //                                                       // or end of line

        // let result = &line[start_bytes..end_bytes]; //slicing line, returns patterndf1老虎23
    }
    Ok(())
}
