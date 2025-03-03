use tokio::net::{TcpStream, TcpListener};
use std::error::Error;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncWriteExt, AsyncReadExt};

const CONTROL: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(CONTROL).await?;

    let args: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(std::env::args().collect()));

    loop {
        let (socket, _) = listener.accept().await?;
        let a = Arc::clone(&args);
        tokio::spawn(async move {

            handle_conn(socket, a).await;

        });
    }
}

async fn handle_conn(mut stream: TcpStream, args: Arc<Mutex<Vec<String>>>) {

    let (mut reader, mut writer) = stream.split();

    // The client will be sending data to the server first
    
    let mut buf = vec![0u8; 1024];

    let n = reader.read(&mut buf).await.expect("Failed to read from reader");
    println!("Message: {}", String::from_utf8_lossy(&buf[..n]));

    let arg;
    {
        arg = args.lock().unwrap().get(1).expect("No Args Provided").clone();
    }

    writer.write_all(arg.as_bytes()).await.expect("Writer failed to write");

    writer.shutdown().await;
}
