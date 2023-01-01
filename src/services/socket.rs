use std::sync::{Arc, mpsc::{channel, Sender}};

use async_std::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::spawn,
};
use async_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use once_cell::sync::OnceCell;
use rand::rngs::OsRng;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    errors::Error,
    methods::{
        ErrorResponse, Event, HelloEvent, Response, RpcApiEvent,
        RpcApiMethod, RpcApiResponse, get_respond,
    },
    services::encryption::{generate, random_number}, globals::HEARTBEAT_TIMEOUT,
};

use super::environment::LISTEN_ADDRESS;

static SERVER: OnceCell<TcpListener> = OnceCell::new();

#[derive(Clone)]
pub struct RpcClient {
    pub(crate) id: String,
    pub(crate) socket: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    pub(crate) user_id: Option<String>,
    pub(crate) request_ids: Arc<Mutex<Vec<String>>>,
    pub heartbeat_tx: Arc<Mutex<Sender<()>>>,
}

impl RpcClient {
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
    connection_loop().await;
}

async fn connection_loop() {
    let clients: DashMap<String, RpcClient> = DashMap::new();
    let mut incoming = SERVER.get().expect("Failed to get server").incoming();
    while let Some(stream) = incoming.next().await {
        let clients = clients.clone();
        spawn(async move {
            let connection = stream.unwrap();
            println!("Socket connected: {}", connection.peer_addr().unwrap());
            let ws_stream = accept_async(connection).await.expect("Failed to accept");
            let (write, mut read) = ws_stream.split();
            let socket_arc = Arc::new(Mutex::new(write));
            let id = generate(
                random_number,
                &[
                    'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p',
                    'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                ],
                10,
            );
            let mut request_ids = Vec::new();
            for _ in 0..20 {
                request_ids.push(generate(
                    random_number,
                    &[
                        'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o',
                        'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
                    ],
                    10,
                ));
            }
            let secret = EphemeralSecret::new(OsRng);
            let public_key = PublicKey::from(&secret);
            let mut buf = Vec::new();
            let val = RpcApiEvent {
                event: Event::Hello(HelloEvent {
                    public_key: public_key.to_bytes().to_vec(),
                    request_ids: request_ids.clone(),
                }),
            };
            val.serialize(&mut Serializer::new(&mut buf).with_struct_map())
                .unwrap();
            // println!("test: {:?}", buf);
            let mut write = socket_arc.lock().await;
            write.send(Message::Binary(buf)).await.unwrap();
            drop(write);

            let (tx, rx) = channel::<()>();
            let clients_moved = clients.clone();
            let id_moved = id.clone();
            spawn(async move {
                while rx.recv_timeout(std::time::Duration::from_millis(*HEARTBEAT_TIMEOUT)).is_ok() {}
                if let Some(client) = clients_moved.get(&id_moved) {
                    let mut socket = client.socket.lock().await;
                    socket.close().await.expect("Failed to close socket");
                }
            });
            let client = RpcClient {
                id: id.clone(),
                socket: socket_arc.clone(),
                user_id: None,
                request_ids: Arc::new(Mutex::new(request_ids)),
                heartbeat_tx: Arc::new(Mutex::new(tx)),
            };
            clients.insert(id.clone(), client);
            while let Some(data) = read.next().await {
                match data.unwrap() {
                    Message::Binary(bin) => {
                        println!("Received binary data");
                        let mut deserializer = Deserializer::new(bin.as_slice());
                        let result: Result<RpcApiMethod, rmp_serde::decode::Error> =
                            Deserialize::deserialize(&mut deserializer);
                        if let Ok(r) = result {
                            println!("Received: {:?}", r.method);
                            if let Some(request_id) = r.id {
                                let client = clients.get(&id).unwrap();
                                let mut request_ids = client.request_ids.lock().await;
                                if request_ids.contains(&request_id) {
                                    request_ids.retain(|x| x != &request_id);
                                } else {
                                    let error = Response::Error(ErrorResponse {
                                        error: Error::InvalidRequestId,
                                    });
                                    let mut value_buffer = Vec::new();
                                    error
                                        .serialize(
                                            &mut Serializer::new(&mut value_buffer)
                                                .with_struct_map(),
                                        )
                                        .unwrap();
                                    let mut write = socket_arc.lock().await;
                                    write.send(Message::Binary(value_buffer)).await.unwrap();
                                    drop(write);
                                    return;
                                }
                                drop(request_ids);
                                drop(client);
                                let dispatch = get_respond(r.method).respond(clients.clone(), id.clone()).await;
                                let mut value_buffer = Vec::new();
                                let return_value = RpcApiResponse {
                                    id: Some(request_id),
                                    response: dispatch,
                                };
                                return_value
                                    .serialize(
                                        &mut Serializer::new(&mut value_buffer)
                                            .with_struct_map(),
                                    )
                                    .unwrap();
                                let mut write = socket_arc.lock().await;
                                write.send(Message::Binary(value_buffer)).await.unwrap();
                                drop(write);
                            } else {
                                let error = Response::Error(ErrorResponse {
                                    error: Error::InvalidRequestId,
                                });
                                let mut value_buffer = Vec::new();
                                error
                                    .serialize(
                                        &mut Serializer::new(&mut value_buffer)
                                            .with_struct_map(),
                                    )
                                    .unwrap();
                                let mut write = socket_arc.lock().await;
                                write.send(Message::Binary(value_buffer)).await.unwrap();
                                drop(write);
                            }
                        } else {
                            let error = Response::Error(ErrorResponse {
                                error: Error::InvalidMethod,
                            });
                            let mut value_buffer = Vec::new();
                            let return_value = RpcApiResponse {
                                id: None,
                                response: error,
                            };
                            return_value
                                .serialize(
                                    &mut Serializer::new(&mut value_buffer).with_struct_map(),
                                )
                                .unwrap();
                            let mut write = socket_arc.lock().await;
                            write.send(Message::Binary(value_buffer)).await.unwrap();
                            drop(write);
                        }
                    }
                    Message::Ping(bin) => {
                        println!("Received ping");
                        let mut write = socket_arc.lock().await;
                        write.send(Message::Pong(bin)).await.unwrap();
                        drop(write);
                    }
                    Message::Close(_) => {
                        println!("Received close");
                        let mut write = socket_arc.lock().await;
                        write.close().await.unwrap();
                        clients.remove(&id.clone());
                        drop(write);
                    }
                    _ => {
                        println!("Received unknown message");
                        let mut write = socket_arc.lock().await;
                        write.close().await.unwrap();
                        clients.remove(&id.clone());
                        drop(write);
                    }
                }
            }
            
        });
    }
}
