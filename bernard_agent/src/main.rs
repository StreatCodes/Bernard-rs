use tokio::prelude::*;
use tokio::net::{TcpStream};
use tokio::timer;
use std::time::Duration;
use sodiumoxide::crypto::secretstream::xchacha20poly1305 as chacha;
use std::vec::Vec;
use tokio::codec::{Framed, LengthDelimitedCodec};
use bernard::{Message, MessageType};
use bernard::check::{Response, CheckHealthResponse};

#[tokio::main]
async fn main() {
    connect_to_parent().await;
}

async fn connect_to_parent() {
    let parent = "127.0.0.1:9359"; 
    let retry_time = 5;

    println!("Connecting to {}", parent);
    let mut stream = loop {
        match TcpStream::connect(parent).await {
            Ok(stream) => break stream,
            Err(e) => {
                println!("Error connecting to parent: {}. Retrying in {} seconds.", e, retry_time);
                timer::delay_for(Duration::from_secs(retry_time)).await;
            }
        };
    };

    let host_id = b"1234567890123456";
    println!("Connected, sending hello");
    stream.write_all(host_id).await.unwrap();

    let (mut encryptor, mut decryptor) = resolve_server_challenge(&mut stream).await.unwrap();
    println!("Read challenge");

    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());

    while let Some(raw_message) = framed.next().await {
        println!("Client received message");
        let raw_message = raw_message.expect("Failed to decode raw message");
        let message = Message::from_encoded_bytes(raw_message, &mut decryptor);

        match message.destination_route.split_first() {
            Some((next, remaining_dests)) => {
                println!("Forwarding message TODO");
            },
            None => {
                match message.message {
                    MessageType::Request(req) => {
                        println!("Recieved request {}", message.id);
                        let response = Message{
                            id: 1,
                            destination_route: Vec::new(),
                            message: 
                                MessageType::Response(
                                    Response::CheckHealth(CheckHealthResponse{})
                                )
                        };

                        let bytes = response.encode_and_encrypt(&mut encryptor);
                        framed.send(bytes.freeze()).await.unwrap();

                    },
                    MessageType::Response(res) => {
                        println!("Received response");
                    }
                }
            }
        }
    }
}

async fn resolve_server_challenge(stream: &mut TcpStream) ->
        Result<(chacha::Stream<chacha::Push>, chacha::Stream<chacha::Pull>), std::io::Error> {
    let mut header_buf = vec![0; chacha::HEADERBYTES];
    stream.read_exact(&mut header_buf).await?;
    
    let mut challenge_buf = vec![0; 128 + chacha::ABYTES];
    stream.read_exact(&mut challenge_buf).await?;
    println!("Solving challenge");

    let header = chacha::Header::from_slice(&header_buf)
        .expect("Failed to read header");

    let priv_key = chacha::Key::from_slice(b"12345678901234567890123456789012")
        .expect("Couldn't load key from bytes");
    let mut decryptor = chacha::Stream::init_pull(&header, &priv_key)
        .expect("Couldn't initialize chacha20");

    let (mut challenge, _) = decryptor.pull(&challenge_buf, None)
        .expect("Couldn't decode challenge");

    challenge.reverse();

    let (mut encryptor, header) = chacha::Stream::init_push(&priv_key)
        .expect("Failed to initialize chacha20 client writer");
    
    let mut solved_challenge = encryptor.push(
        &challenge,
        None,
        chacha::Tag::Message
    ).expect("Failed to encrypt solved challange");
    
    println!("Sending client header");
    stream.write_all(&mut header.as_ref().to_vec()).await?;
    println!("Sending solved challenge");
    stream.write_all(&mut solved_challenge).await?;

    Ok((encryptor, decryptor))
}