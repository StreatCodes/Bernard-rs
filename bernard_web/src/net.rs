use sodiumoxide::crypto::secretstream::xchacha20poly1305 as chacha;
use std::collections::HashMap;
use tokio::prelude::*;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, Sender, Receiver};
use std::pin::{Pin};
use std::task::{Context, Poll};
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use bytes::BytesMut;
use std::sync::atomic::{AtomicU64, Ordering};
use bernard::{HostId, Message, MessageType};
use bernard::check::{Request};
use futures::stream::{Stream};
use futures_util::stream::StreamExt;
use futures_util::sink::Sink;

pub struct ClientManager {
    message_count: AtomicU64,
    clients: Mutex<HashMap<HostId, Client>>,
}

pub struct Client {
    sender: Sender<Message>
}

impl ClientManager {
    pub fn new() -> Self {
        ClientManager {
            message_count: AtomicU64::new(0),
            clients: Mutex::new(HashMap::new())
        }
    }
    pub async fn add_client(
        &self,
        host_id: HostId,
        stream: TcpStream,
        encryptor: chacha::Stream<chacha::Push>,
        mut decryptor: chacha::Stream<chacha::Pull>
    ) {
        let (sender, receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel(2048);

        self.clients.lock().await.insert(
            host_id,
            Client {
                sender: sender
            }
        );

        let framed = Framed::new(stream, LengthDelimitedCodec::new());

        let mut message_handler = MessageHandler {
            stream: framed,
            encryptor: encryptor,
            receiver: receiver
        };

        tokio::spawn(async move {
            while let Some(result) = message_handler.next().await {
                match result {
                    Ok(bytes) => {
                        let message = Message::from_encoded_bytes(bytes, &mut decryptor);
                        println!("Message received");
                        println!("{:#?}", message);
                    },
                    Err(e) => {println!("Error")}
                }
            }

            println!("{:?} Disconnected.", host_id);
        });
    }

    pub async fn send_message(&self, host_id: HostId, request: Request) {
        let message_id = self.message_count.fetch_add(1, Ordering::Relaxed);
        let dest = vec![];
        
        let message = Message {
            id: message_id,
            destination_route: dest,
            message: MessageType::Request(request)
        };

        println!("Sending message");
        let mut clients = self.clients.lock().await;
        let sender = clients.get_mut(&host_id).expect("Couldn't find host in map");
        sender.sender.send(message).await.expect("Failed to queue message");
    }
}

struct MessageHandler {
    stream: Framed<TcpStream, LengthDelimitedCodec>,
    encryptor: chacha::Stream<chacha::Push>,
    receiver: Receiver<Message>
}

impl Stream for MessageHandler {
    type Item = Result<BytesMut, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Poll::Pending = Pin::new(&mut self.stream).poll_ready(cx) {
            return Poll::Pending;
        }

        //Poll for mpsc messages
        if let Poll::Ready(Some(message)) = self.receiver.poll_next_unpin(cx) {
            println!("Sending message");
            let bytes = message.encode_and_encrypt(&mut self.encryptor);
            Pin::new(&mut self.stream).start_send(bytes.freeze()).expect("Failed to start sending");

            match Pin::new(&mut self.stream).poll_flush(cx) {
                Poll::Pending => {return Poll::Pending},
                Poll::Ready(res) => {println!("Message flushed");}
            }
        }

        //Poll for tcp stream frames
        if let Poll::Ready(raw_message) = self.stream.poll_next_unpin(cx) {
            println!("Received message in poll");
            let bytes = match raw_message {
                Some(res) => res.unwrap(),
                None => return Poll::Ready(None)
            };
            return Poll::Ready(Some(Ok(bytes)))
        }

        Poll::Pending
    }
}