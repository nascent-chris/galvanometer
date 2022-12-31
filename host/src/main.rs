use std::{io::Write, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use coingecko::CoinGeckoClient;
    let client = CoinGeckoClient::default();

    let port_name = "/dev/ttyACM0";

    // the port name is any /dev/ttyACM* device

    let baud_rate = 1_000_000;

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(Duration::from_millis(2000))
        .open()?;

    let mut serial_buf: Vec<u8> = vec![0; 2048];

    loop {
        let response = client
            .price(&["chainlink"], &["usd"], true, true, true, true)
            .await;

        // println!("{:?}", response);

        let Ok(response) = response else {
            println!("Error getting price: {:?}", response);
            continue;
        };

        let Some(price) = response.get("chainlink").and_then(|res| res.usd) else {
            println!("Error getting price: {:?}", response.get("chainlink"));
            // write response as json to debug file
            std::fs::File::create("debug.json")?.write_all(serde_json::to_string(&response)?.as_bytes())?;
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        };

        println!("{:?}", price);

        // scale the price from 4-7 to 0-255
        let percentage = 0xFF as f64 * (price - 4.0) / 3.0;

        let res = port
            .write(&[percentage as u8])
            .map_err(|e| anyhow::anyhow!("Error writing to serial port: {:?}", e));

        let Ok(written_bytes) =  res else {
                println!("{:?}", res);
                continue;
            };

        let res = port.read(serial_buf.as_mut_slice());
        let Ok(bytes_written) = res else {
            println!("Error reading from serial port: {:?}", res);
            continue;
        };

        println!("Sent: 0x{}", hex::encode(&[percentage as u8]));
        println!("Rcv: 0x{}", hex::encode(&serial_buf[..bytes_written]));
        tokio::time::sleep(Duration::from_secs(10)).await;
    }

    Ok(())
}
