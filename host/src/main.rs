use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port_name = "/dev/ttyACM0";
    let baud_rate = 1_000_000;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(2000))
        .open()?;

    let mut serial_buf: Vec<u8> = vec![0; 2048];
    // let written_bytes = port.write(&[0x01, 0x02, 0x03])?;
    let written_bytes = port.write(&[0xFF / 3])?;

    let data = port.read(serial_buf.as_mut_slice())?;
    println!("Read {} bytes", data);
    println!("Data: {:?}", hex::encode(&serial_buf[..data]));

    Ok(())
}
