mod net;

use tokio::prelude::*;
use tokio::net::{TcpListener, TcpStream};
use sodiumoxide::crypto::secretstream::xchacha20poly1305 as chacha;
use rand::prelude::*;
use rand::RngCore;
use rand_chacha::ChaCha20Rng;
use std::vec::Vec;
use bernard::error::Error;
use bernard::{HostId};
use bernard::check::{Request, CheckHealthRequest};
use net::{ClientManager};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_manager = Arc::new(ClientManager::new());
    let mut listener = TcpListener::bind("127.0.0.1:9359").await?;

    loop {
        println!("waiting for conns");
        let (mut stream, _) = listener.accept().await?;
        println!("new conn");

        let client_manager = Arc::clone(&client_manager);

        tokio::spawn(async move {
            let (host_id, encryptor, solved_challenge) = challenge_client(&mut stream).await;
            let client_res = verify_client_challenge(&mut stream, solved_challenge).await;

            let decryptor = match client_res {
                Ok(decryptor) => decryptor,
                Err(e) => {println!("Client failed to connect {}", e); return ()},
            };

            client_manager.add_client(host_id, stream, encryptor, decryptor).await;

            client_manager.send_message(host_id, Request::CheckHealth(CheckHealthRequest{})).await;
        });
    }
}

async fn challenge_client(stream: &mut TcpStream) -> (HostId, chacha::Stream<chacha::Push>, Vec<u8>) {
    println!("Waiting for client hello");
    let mut host_id: HostId = [0; 16];
    stream.read_exact(&mut host_id).await
        .expect("Couldn't read client hello");
    println!("Received hello");
    
    let mut challenge_buf = vec![0; 128];
    let mut rng = ChaCha20Rng::from_entropy();
    rng.fill_bytes(&mut challenge_buf);

    let priv_key = b"12345678901234567890123456789012";
    let priv_key = chacha::Key::from_slice(priv_key).expect("Invalid key");
    let (mut encryptor, header) = chacha::Stream::init_push(&priv_key)
        .expect("Failed to initialize chacha20 writer");
    
    let mut encrypted_challenge = encryptor.push(
        &challenge_buf,
        None,
        chacha::Tag::Message
    ).expect("Failed to encrypt challange");

    println!("Sending header");
    stream.write_all(&mut header.as_ref().to_vec()).await
        .expect("Failed to send header to client challenge");
    println!("Sending challenge");
    stream.write_all(&mut encrypted_challenge).await
        .expect("Failed to send challenge to client challenge");

    challenge_buf.reverse();

    (host_id, encryptor, challenge_buf)
}

async fn verify_client_challenge(stream: &mut TcpStream, solved_challenge: Vec<u8>)
    -> Result<chacha::Stream<chacha::Pull>, Error> {
    println!("waiting for client to resolve challenge");
    let mut header_buf = vec![0; chacha::HEADERBYTES];
    stream.read_exact(&mut header_buf).await?;

    let mut challenge_buf = vec![0; 128 + chacha::ABYTES];
    stream.read_exact(&mut challenge_buf).await?;

    let header = chacha::Header::from_slice(&header_buf)
        .expect("Couldn't load header from bytes");

    let priv_key = chacha::Key::from_slice(b"12345678901234567890123456789012")
        .expect("Couldn't load key from bytes");
    let mut decryptor = chacha::Stream::init_pull(&header, &priv_key)
        .expect("Couldn't initialize chacha20");

    let (client_challenge, _) = decryptor.pull(&challenge_buf, None)
        .expect("Couldn't decode challenge");

    if client_challenge != solved_challenge {
        Err(Error::new(String::from("Client failed challenge")))
    } else {
        println!("Client succeeded");
        Ok(decryptor)
    }
}