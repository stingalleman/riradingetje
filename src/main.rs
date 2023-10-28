use std::{env, io::Read, time::Duration};

fn main() {
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| "/dev/ttyUSB0".into());

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

        println!("{:?}", state.slaves.len());
    }
}
