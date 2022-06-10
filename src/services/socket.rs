use std::sync::Arc;

use async_std::{
    net::{TcpListener, TcpStream},
    prelude::StreamExt,
    sync::Mutex,
    task::spawn,
};
use async_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use dashmap::DashMap;
use futures_util::SinkExt;
use once_cell::sync::OnceCell;
use rand_core::OsRng;
use rmp_serde::{decode::Error, Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    methods::voice::{
        self, ErrorResponse, Event, HelloEvent, Method, Response, VoiceApiEvent, VoiceApiMethod,
        VoiceApiResponse,
    },
    services::encryption::{generate, random_number},
};

use super::environment::LISTEN_ADDRESS;

static SERVER: OnceCell<TcpListener> = OnceCell::new();

#[derive(Clone)]
pub struct VoiceClient {
    pub(crate) id: String,
    pub(crate) socket: Arc<Mutex<WebSocketStream<TcpStream>>>,
    pub(crate) user_id: Option<String>,
}

impl VoiceClient {
    pub fn get_user_id(&self) -> String {
        match &self.user_id {
            Some(v) => v.to_string(),
            None => "".to_string(),
        }
    }
}

pub async fn start_server() {
    let server = TcpListener::bind(LISTEN_ADDRESS.to_owned()).await.unwrap();
    SERVER.set(server).expect("Failed to set server");
    spawn(connection_loop());
}

async fn connection_loop() {
    let clients = Arc::new(Mutex::new(DashMap::new()));
    let mut incoming = SERVER.get().expect("Failed to get server").incoming();
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
            let client = VoiceClient {
                id: id.clone(),
                socket: socket_arc.clone(),
                user_id: None,
            };
            clients_arc.lock().await.insert(id.clone(), client);
            let mut socket = socket_arc.lock().await;
            let secret = EphemeralSecret::new(OsRng);
            let public_key = PublicKey::from(&secret);
            let mut buf = Vec::new();
            let val = VoiceApiEvent {
                data: Some(Event::Hello(HelloEvent {
                    public_key: public_key.to_bytes().to_vec(),
                })),
            };
            val.serialize(&mut Serializer::new(&mut buf)).unwrap();
            socket.send(Message::Binary(buf)).await.unwrap();
            loop {
                while let Some(data) = socket.next().await {
                    match data.unwrap() {
                        Message::Binary(bin) => {
                            println!("Received binary data");
                            let mut deserializer = Deserializer::new(bin.as_slice());
                            let result: Result<VoiceApiMethod, Error> =
                                Deserialize::deserialize(&mut deserializer);
                            if let Ok(r) = result {
                                let data = r.data.unwrap();
                                println!("Received: {:?}", data);
                                let dispatch: Response = match data {
                                    Method::Identify(m) => {
                                        voice::authentication::identify(
                                            m,
                                            clients_arc.clone(),
                                            id.clone(),
                                        )
                                        .await
                                    }
                                    Method::Capabilities(m) => {
                                        voice::webrtc::capabilities(m).await
                                    }
                                    Method::Transport(m) => {
                                        voice::webrtc::transport(
                                            m,
                                            clients_arc.clone(),
                                            id.clone(),
                                        )
                                        .await
                                    }
                                    Method::Dtls(m) => voice::webrtc::dtls(m).await,
                                    Method::Produce(m) => voice::webrtc::produce(m).await,
                                    Method::Consume(m) => voice::webrtc::consume(m).await,
                                    Method::Resume(m) => voice::webrtc::resume(m).await,
                                };
                                let mut value_buffer = Vec::new();
                                let return_value = VoiceApiResponse {
                                    id: None,
                                    data: Some(dispatch),
                                };
                                return_value
                                    .serialize(&mut Serializer::new(&mut value_buffer))
                                    .unwrap();
                                socket.send(Message::Binary(value_buffer)).await.unwrap();
                            } else {
                                let error = Response::Error(ErrorResponse {
                                    error: "Invalid data or method not found".to_string(),
                                });
                                let mut value_buffer = Vec::new();
                                let return_value = VoiceApiResponse {
                                    id: None,
                                    data: Some(error),
                                };
                                return_value
                                    .serialize(&mut Serializer::new(&mut value_buffer))
                                    .unwrap();
                                socket.send(Message::Binary(value_buffer)).await.unwrap();
                            }
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
