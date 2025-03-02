use tokio::net::{TcpStream, TcpListener};
use std::error::Error;
use tokio::time::{sleep, Duration};

const CONTROL: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    /// For now, lets just establish a reverse connection to the control server

    let mut stream = reconnect().await; 

    let mut msg = vec![0; 1024];

    loop {
        
        stream.readable().await?;

        match stream.try_read(&mut msg) {
            Ok(0) => {
                sleep(Duration::from_millis(5000));
                continue;
            } 
            Ok(n) => {
                msg.truncate(n);
                break;
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            },
            Err(_) => {
                stream = reconnect().await
            }
        }

        println!("{:?}", msg);
    }

    Ok(())
}

// Is used to reconnect (or start trying) to get back
// a connection with the control server
async fn reconnect() -> TcpStream {
    loop {
        match TcpStream::connect(CONTROL).await {
            Ok(stream) => return stream,
            Err(_) => (),
        }

        sleep(Duration::from_millis(5000)).await;
    }
}
