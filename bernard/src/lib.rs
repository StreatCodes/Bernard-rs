use sodiumoxide::crypto::secretstream::xchacha20poly1305 as chacha;
use serde::{Serialize, Deserialize};
use bytes::BytesMut;
use check::{Request, Response};

pub mod error;
pub mod check;

pub type HostId = [u8; 16];
pub type DestinationRoute = Vec<[u8; 16]>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub id: u64,
    pub destination_route: DestinationRoute,
    pub message: MessageType
}

impl Message {
    pub fn encode_and_encrypt(self, encryptor: &mut chacha::Stream<chacha::Push>) -> BytesMut {
        let raw_message: Vec<u8> = bincode::serialize(&self)
            .expect("Failed to serialize request");

        let encrypted_message = encryptor.push(
            &raw_message,
            None,
            chacha::Tag::Message
        ).expect("Failed to encrypt message");

        BytesMut::from(&encrypted_message[..])
    }
    pub fn from_encoded_bytes(bytes: BytesMut, decryptor: &mut chacha::Stream<chacha::Pull>) -> Self {
        let (raw_message, _) = decryptor.pull(&bytes, None)
            .expect("Couldn't decode message");

        let message: Message = bincode::deserialize(&raw_message[..])
            .expect("Failed to decode binary message");

        message
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Request(Request),
    Response(Response)
}