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

use rand;

const CONTROL: &str = "127.0.0.1:8080";
const PRIVATE_KEY_PEM: &str = "-----BEGIN RSA PRIVATE KEY-----
MIIEpgIBAAKCAQEA3Q4YyAa124SngD/gET8G+2y5tECs0DID/M2TftoLPcnKlfpi
ZOSPS2bz9bWA33g0NgE3wGBFtYWevQVMwdU18HR6V5BF1nkmF9JOOHGzcTPXTFHa
UKPVh13+njkQhuEk8y0wNDSz58mEdHAKL/ckyMFl2o1Z5qrAoGO0+v6stUcD/nrr
bdvosfkJmUo3aVQTy2avtX94h8tp1uLp6aFV/5IvegW3FLAYMbDHkCV8VPzwRVl5
mSLQIrQKfht+L7Kb7aWTKxcDS+o2oHRIpZWSlE3WvG4xHNp0lhiSJs3i+m0loTqu
efnewanT2eKi+DC7sokMR16nvIyJGz0TGISzNQIDAQABAoIBAQC5Bzum8jb2b6rk
ppKe3OqzOOaYAiqkebjsugZjf485QNbZgqMpYEaxDpPVbv9YDf1l1s8br3mp3xZn
kTAWbqifhF+DO7e5Nd60wcvHYySX9JSZxUm/heMNU5f0SD+VJo8PHRvkFtkfgoXk
EIP2ch/QClF/BiHh+bsShWLPVKyrbh+kRrioi5v8wBGyaKo9NXAjygdF7ATnZctb
w9Y6r9M/APZftaE0TKW2/WvnpWbnHE8x4mktN6e5IsjEKClcDTFUA6d7YsFhHW3M
cCYcwRWRBGp/eK+82NSZld2unO0Z+Gs5CYFmNUQIpxm16ta5oqPcdw2MZ9KqOah7
NwPBXiZhAoGBAO2FUoSwXGgj++WitCB+un3nA6NxmQ4rBPNVN5N0jqaSvdwqyudG
iWSnj1GzmWK3ONo3AzEfVxV6C3G1aTmQd6jEzYhC14/c/T1PyCwOh7NMe5lNEFvT
nLQ87b9PsSM/wWruJsoGK3qbMJtBydtXZobOs57lGafopjmLwINRYf5/AoGBAO5A
1AphCfQHAdCCinT4kH0FmqlUf5WKwh6KQogUzCIohpIaMlUnWjaHKqsdF4MtE52f
5S8Nj3LmIttuKCS4EHBCsXXMnDQ1Zj16fwALPd9+p+L+YO+Bk2OZXtVbrG475U/E
kLnuHnZhCT67GE9kMLNOtSlFMtJrfkkKp3ac5txLAoGBAMyZLJS56k+Wt5R+i9T1
9SUgywSOhP4sVm5ZOK9fFsYr3zn47qE8JHu8znrdT0I2e9WUz74Ptg78+0CuuP5q
EHkjVbyLf24gOVAIGnDA72mmd019kZrX9MNZPJ6wQfEQKUc379fUar2JUtxaVkwH
xHxZL2avCGOXxyAI3wL/Jn93AoGBAMBt09TeM2/JCQojpTh8WhbgI96HxNQ/UjHg
cYP/aDBekvIF/okQU6ie2zOBN7BXA/TXY//w799ZSOX5vqqJj51cAICoVbFFP+J/
wXuDmOGeJnMMCbbn7OcQxKuGc2pFLhyTImFJlYj/Lrxf2G6m+T6ZX3CJ11ZkQ4cy
97axWzSpAoGBAIkm5UaCxstydRrxJr5VfLEAR7ey2BhsoFGgDC3UkCUZ+omODJd3
Iu3kriS1o/1RbuWsOkrq80TfuDvSdB74tN9wCcqVRHNYV2IjW9XAe2+P0LiJCHz7
cq+mnghyfeaFOHgcwZSCMDW5oNDC8QleSzQsWwNCDENlfEfYC+Cju7fd
-----END RSA PRIVATE KEY-----";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(CONTROL).await?;

    let args: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(std::env::args().collect()));

    let private_key = RsaPrivateKey::from_pkcs1_pem(PRIVATE_KEY_PEM).expect("Could not get private key");
    let signing_key: Arc<Mutex<SigningKey<Sha256>>> = Arc::new(Mutex::new(SigningKey::<Sha256>::new(private_key)));

    loop {
        let (socket, _) = listener.accept().await?;
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
