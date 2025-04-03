use tokio::net::{TcpStream, TcpListener};
use std::error::Error;
use std::sync::{Arc, Mutex};
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

const CONTROL: &str = "0.0.0.0:8080";
const PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA1KBXWBk/GxsfTWhKTSUp7SB/M+cE9vQHVYaeRgtzN4t430Jb
mUbU3bMVusTc5FzE659d7JFpFmEV1jQuuscwarBtG4LTsgRzm/WntGlXLXqCLlr/
lka01f5Lc+myAWj+fVGmFJArC20cdo2KEJXVnQnm9VYzUI0yz9Y0lyvBJYCoYqoX
mX+ZJE6D/VrgZY5XSmeiDUeV5W8+wFUj7VE89B9agzKXjz3K1xUsrz7Q5Ri9XQ+F
7ouLghP0P975jWSP2sHbRdtXSei7zb4baGOiOe/7mH1/xUSGmxyHuQlc0haOGUXC
7jxT2jefOObS8Jry+lGlMMYpWSa5YazWBF+wnwIDAQABAoIBAHz0+j/hHnAcoWnJ
d5kc1SXxajAjfYxwX3sI0S1S/1ROWCvFALX0pZbg701C8poWVvguQXGNB1hKmB25
rxi/hRm+M3DukuDUsuieFv/0xHIKxETf38L+LLpyZv9pAN7JSH53pIYDL/jYF9ko
V1Jl7BMNRbk6/o9BW/I+7ctFwI37mi4iXwgOtYbhqd97MiNMrw4RuvZskhnUhTnz
5L6piNdPASxb2eJSb5DzmIG7xr8AwVoFB86ErPWyCsE4LT258fNQjU+RcCBeMQP+
rxvnDBN3dcsYDWcNRBBjE+OyEgBnxZNuxHIPKVhiqfyqCti485a8gDMIF1JCTNEf
VTL5Q7ECgYEA/ti4kRVmGzFXhDN0/KTwOItmF7XvlnyZRJ6kO4K8nZIVk65QP9aS
KPdDjrX2KRoiCRAzz87O+8oIrbnjoIvPuUMc+oqy3RVQDDqNlypKNC8A8ID87j46
FaO6RbccV1upKGsht7NqkIz+sQMtdF4E7hrSA9gxcdUk0j9xr+kfyDcCgYEA1Zaz
mmZ4VKs0hD6OCr0wabdxhKaJTnhWr0Cw1IMhv/rO8WiHSzDqNKbys50Jxh3nwN01
xrQ41SzT3yocWjkiQA+6TYf3aAchZohPjmbNlXRf2bWW/chY8FISP7qsQCuXzAhI
KcXVFWYYMVcH5A+iA62/OGIrlt7/WZn2/ab31tkCgYEAmw287fj5EN5qgvLT4Mtx
pNWbnh2B0iupDQkAn3yhybUSD7Jk/RJOB0K6BeBMDyHpR15mR69qW+PGd/y7s/ic
v/lHbmBpBv5qn6YYc7q1Px7CEZJJUaLmsDfXo4cPZU9NklvsPdqZoa1HYOewBjcW
BNCoqtIk8z9dkdYsE31hau0CgYAtcpCq5OzrzzYwgYMF4/W+OOttt2X2DYMcbV5z
CagPypVoJr7LEBB1vaRc9ahTwJ9EmY1Mx6JkMdKvZK19wTR5fx88ShcpyoKf55dO
DK/oAkg28f276HqMQFpjtxvZ16zIHGcGhHLvx7aUIRmrvGvT7o+GttThmHerICi/
Fl0cGQKBgAfnceKHOuz/tQ8in1Wb17O0Mt+m0z9/lUc69JV4mSur/2MSjjKsB1Mu
N+sXt8QolJCKjIn0/pKnDNcY1ZPwpfwFHQ77Mg6OlItW4i7CBpQoP5CVPOaObwlg
hNRAhMwvEJWBcTVrjAhWACOT3KtCAZojlluwnaNSJqnfJBjn/4Zl
-----END RSA PRIVATE KEY-----";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(CONTROL).await?;
    let mut unique_connections: HashSet<SocketAddr> = HashSet::new();

    let args: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(std::env::args().collect()));

    let private_key = RsaPrivateKey::from_pkcs1_pem(PRIVATE_KEY_PEM).expect("Could not get private key");
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
