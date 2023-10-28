use std::time::Duration;

fn main() {
    println!("running");
    let mut port = serialport::new("/dev/ttyUSB0", 115_200)
        .timeout(Duration::from_millis(10000))
        .parity(serialport::Parity::Even)
        .stop_bits(serialport::StopBits::One)
        .data_bits(serialport::DataBits::Seven)
        .open()
        .expect("Failed to open port");

    let mut serial_buf: Vec<u8> = vec![0; 2048];
    port.read(serial_buf.as_mut_slice())
        .expect("Found no data!");

    // println!("{}", String::from_utf8(serial_buf).unwrap());
    println!("{:?}", serial_buf)
}
