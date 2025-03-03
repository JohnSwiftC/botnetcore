use tokio::net::{TcpStream};
use std::error::Error;
use tokio::time::{Duration, sleep};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

const CONTROL: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   // We ideally want to do the following things here
   // 1. Connect to the server if not already connection
   // 2. Send a ping message to the server
   // 3. Recieve command from the control server
   // 4. Sleep to slow down requests

    loop {
        let mut stream = get_stream().await;
        let (mut reader, mut writer) = stream.split();
        if let Err(_) = writer.write_all(b"Hello!").await {
            writer.shutdown().await;
            continue;
        }
            
        let mut buf = vec![0u8; 1024];

        if let Err(_) = reader.read(&mut buf).await {
            writer.shutdown().await;
        }

        println!("Message from control: {}", String::from_utf8_lossy(&buf[..]));

        writer.shutdown().await;
        
        sleep(Duration::from_millis(5000)).await;
    }
}

async fn get_stream() -> TcpStream {
    loop {
        match TcpStream::connect(CONTROL).await {
            Ok(s) => return s,
            Err(_) => {
                sleep(Duration::from_millis(5000));
            }
        }
    }
}
