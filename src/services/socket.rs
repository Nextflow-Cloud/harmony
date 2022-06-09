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
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    methods::{
        self, Event, HelloEvent, Method, NotFoundResponse, Response, RpcApiEvent, RpcApiMethod,
        RpcApiResponse,
    },
    services::encryption::{generate, random_number},
};

use super::environment::LISTEN_ADDRESS;

static SERVER: OnceCell<TcpListener> = OnceCell::new();

struct RpcClient {
    id: String,
    socket: Arc<Mutex<WebSocketStream<TcpStream>>>,
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
                                Method::Capabilities(m) => {
                                    methods::webrtc::capabilities(socket_arc.clone(), m).await
                                }
                                Method::Transport(m) => methods::webrtc::transport(m).await,
                                Method::Dtls(m) => methods::webrtc::dtls(m).await,
                                Method::Produce(m) => methods::webrtc::produce(m).await,
                                Method::Consume(m) => methods::webrtc::consume(m).await,
                                Method::Resume(m) => methods::webrtc::resume(m).await,
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
