#![feature(arbitrary_enum_discriminant)]

pub mod globals;
pub mod methods;
pub mod services;

use services::database;
use services::environment::{JWT_SECRET, LISTEN_ADDRESS};
use services::webrtc;

use dashmap::DashMap;
// use std::collections::HashMap;
// use std::io::Write;
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::Mutex;
use async_std::task::spawn;
use async_tungstenite::tungstenite::Message;
use async_tungstenite::{accept_async, WebSocketStream};
use futures_util::*;
use std::sync::Arc;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rand_core::OsRng;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};
// use flate2::read::;

// use log::info;

// use mongodb::{bson::doc,};

use crate::methods::{
    Event, HelloEvent, Method, NotFoundResponse, Response, RpcApiEvent, RpcApiMethod,
    RpcApiResponse,
};
// use jsonwebtoken::{decode, encode, Header, Validation};

struct RpcClient {
    id: String,
    socket: Arc<Mutex<WebSocketStream<TcpStream>>>,
}

pub fn random_number(size: usize) -> Vec<u8> {
    let mut rng = StdRng::from_entropy();
    let mut result: Vec<u8> = vec![0; size];
    rng.fill(&mut result[..]);
    result
}

pub fn generate(random: fn(usize) -> Vec<u8>, alphabet: &[char], size: usize) -> String {
    assert!(
        alphabet.len() <= u8::max_value() as usize,
        "The alphabet cannot be longer than a `u8` (to comply with the `random` function)"
    );
    let mask = alphabet.len().next_power_of_two() - 1;
    let step: usize = 8 * size / 5;
    let mut id = String::with_capacity(size);
    loop {
        let bytes = random(step);
        for &byte in &bytes {
            let byte = byte as usize & mask;
            if alphabet.len() > byte {
                id.push(alphabet[byte]);
                if id.len() == size {
                    return id;
                }
            }
        }
    }
}

// pub fn encode(object: RpcApiMethod, compress: bool, encrypt: bool) -> Vec<u8> {

// }

#[async_std::main]
async fn main() {
    database::connect().await;
    println!("Database is connected");
    webrtc::create_workers().await;
    let clients = Arc::new(Mutex::new(DashMap::new()));
    let server = TcpListener::bind(LISTEN_ADDRESS.to_owned()).await.unwrap();
    println!("Server is running on port 9000");
    let mut incoming = server.incoming();
    while let Some(stream) = incoming.next().await {
        let clients_arc = clients.clone();
        spawn(async move {
            let connection = stream.unwrap();
            println!("Socket connected: {}", connection.peer_addr().unwrap());
            let socket_arc = Arc::new(Mutex::new(
                accept_async(connection).await.expect("Failed to accept"),
            ));
            let id = generate(
                random_number,
                &[
                    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                ],
                10,
            );
            let client = RpcClient {
                id: id.clone(),
                socket: socket_arc.clone(),
            };
            clients_arc.lock().await.insert(id.clone(), client);
            let mut socket = socket_arc.lock().await;
            let secret = EphemeralSecret::new(OsRng);
            let public_key = PublicKey::from(&secret);
            let mut buf = Vec::new();
            let val = RpcApiEvent {
                data: Some(Event::Hello(HelloEvent {
                    public_key: public_key.to_bytes().to_vec(),
                })),
                // id: None,
                // error: None,
            };
            val.serialize(&mut Serializer::new(&mut buf)).unwrap();
            socket.send(Message::Binary(buf)).await.unwrap();
            loop {
                while let Some(data) = socket.next().await {
                    match data.unwrap() {
                        Message::Binary(bin) => {
                            println!("Received binary data");
                            let mut deserializer = Deserializer::new(bin.as_slice());
                            let result: RpcApiMethod = Deserialize::deserialize(&mut deserializer)
                                .expect("Failed to deserialize");
                            let data = result.data.unwrap();
                            println!("Received: {:?}", data);
                            let dispatch: Response = match data {
                                Method::Identify(m) => {
                                    methods::authentication::identify(socket_arc.clone(), m)
                                }
                                _ => {
                                    fn not_found(
                                        _: Arc<Mutex<WebSocketStream<TcpStream>>>,
                                    ) -> Response {
                                        println!("Method not found");
                                        Response::NotFound(NotFoundResponse {
                                            error: "Method not found".to_string(),
                                        })
                                    }
                                    not_found(socket_arc.clone())
                                }
                            };
                            let mut value_buffer = Vec::new();
                            let return_value = RpcApiResponse {
                                id: None,
                                error: None,
                                data: Some(dispatch),
                            };
                            return_value
                                .serialize(&mut Serializer::new(&mut value_buffer))
                                .unwrap();
                            socket.send(Message::Binary(value_buffer)).await.unwrap();
                        }
                        Message::Ping(bin) => {
                            println!("Received ping");
                            socket.send(Message::Pong(bin)).await.unwrap();
                        }
                        Message::Close(_) => {
                            println!("Received close");
                            socket.close(None).await.unwrap();
                            clients_arc.lock().await.remove(&id.clone());
                            break;
                        }
                        _ => {
                            println!("Received unknown message");
                            socket.close(None).await.unwrap();
                            clients_arc.lock().await.remove(&id.clone());
                            break;
                        }
                    }
                }
            }
        });
    }
}
