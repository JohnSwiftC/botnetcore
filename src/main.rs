use tokio::net::{TcpStream, TcpListener};
use std::error::Error;
use std::sync::{Arc, Mutex};

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

async fn handle_conn(stream: TcpStream, args: Arc<Mutex<Vec<String>>>) {

    let mut msg = vec![0, 1024];

    loop {

        match stream.try_write(b"This is the botnet control server") {
            Ok(_) => break,
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            },
            Err(e) => panic!("{}", e),
        }


    }
}
