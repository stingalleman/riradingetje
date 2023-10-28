// use futures::stream::StreamExt;
// use std::{ascii, env, io, str};
// use tokio_util::codec::Decoder;

// use bytes::BytesMut;
// use tokio_serial::SerialPortBuilderExt;

// #[derive(Debug)]
// struct DSMR {
//     version: u8,
//     // YYMMDDhhmm, ssX
//     timestamp: u64,
//     delivered_1: f64,
//     delivered_2: f64,
//     tarief: u8,
//     delivered: f64,
//     voltage: f32,
//     current: f32,
//     instantaneous_power: f64,
// }
// struct LineCodec;

// impl Decoder for LineCodec {
//     type Item = String;
//     type Error = io::Error;

//     fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
//         let newline = src.as_ref().iter().position(|b| *b == b'\n');
//         if let Some(n) = newline {
//             let line = src.split_to(n + 1);
//             return match str::from_utf8(line.as_ref()) {
//                 Ok(s) => Ok(Some(s.to_string())),
//                 Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Invalid String")),
//             };
//         }
//         Ok(None)
//     }
// }

// fn get_value(str: String) -> String {
//     let start = str.find("(").unwrap() + 1;
//     let end = str.find(")").unwrap();
//     let res = &str[start..end];

//     res.to_string()
// }

// #[tokio::main]
// async fn main() -> tokio_serial::Result<()> {
//     let mut args = env::args();
//     let tty_path = args.nth(1).unwrap_or_else(|| "/dev/ttyUSB0".into());

//     let mut port = tokio_serial::new(tty_path, 115200)
//         .data_bits(tokio_serial::DataBits::Eight)
//         .stop_bits(tokio_serial::StopBits::One)
//         .parity(tokio_serial::Parity::None)
//         .open_native_async()?;

//     #[cfg(unix)]
//     port.set_exclusive(false)
//         .expect("Unable to set serial port exclusive to false");

//     let mut reader = LineCodec.framed(port);

//     let mut buf: DSMR = DSMR {
//         version: 0,
//         timestamp: 0,
//         delivered_1: 0.0,
//         delivered_2: 0.0,
//         tarief: 0,
//         delivered: 0.0,
//         voltage: 0.0,
//         current: 0.0,
//         instantaneous_power: 0.0,
//     };

//     while let Some(line_result) = reader.next().await {
//         let line = line_result.expect("Failed to read line");

//         // if line.len() == 0 || line.contains(char::is_whitespace) {
//         //     println!("whitespace, {}", line.contains(char::is_whitespace));
//         //     continue;
//         // }

//         if line.contains("/") {
//             buf = DSMR {
//                 version: 0,
//                 timestamp: 0,
//                 delivered_1: 0.0,
//                 delivered_2: 0.0,
//                 tarief: 0,
//                 delivered: 0.0,
//                 voltage: 0.0,
//                 current: 0.0,
//                 instantaneous_power: 0.0,
//             };
//         } else if line.contains("!") {
//             // println!("finished");
//             println!("{:?}", buf);
//             continue;
//         }

//         // println!("{} - {}", line, line.len());
//         // let parameter = &line[0..line.find("(").unwrap()];
//         // println!("{}", parameter);

//         if line.contains("0-0:1.0.0") {
//             let x = get_value(line).strip_suffix("S").unwrap().to_string();
//             buf.timestamp = x.parse().unwrap();
//         }
//     }
//     Ok(())
// }

use std::io::Read;

fn main() {
    let mut port = serial::open("/dev/ttyUSB0").unwrap();
    let reader = dsmr5::Reader::new(port.bytes());

    for readout in reader {
        let x = readout.unwrap();
    }

    // for readout in reader {
    //     let x = readout.unwrap().to_telegram().unwrap();
    //     let state = dsmr5::Result::<dsmr5::state::State>::from(&x).unwrap();

    //     println!("{}", state.power_delivered.unwrap());
    // }
}
