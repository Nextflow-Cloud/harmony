use std::sync::Arc;

use async_std::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::spawn,
};
use async_tungstenite::{accept_async, tungstenite::Message, WebSocketStream};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use once_cell::sync::OnceCell;
use rand_core::OsRng;
use rmp_serde::{decode::Error, Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    methods::{
        authentication, messages, webrtc, ErrorResponse, Event, HelloEvent, Method, Response, RpcApiEvent,
        RpcApiMethod, RpcApiResponse,
    },
    services::encryption::{generate, random_number},
};

use super::environment::LISTEN_ADDRESS;

static SERVER: OnceCell<TcpListener> = OnceCell::new();

#[derive(Clone)]
pub struct VoiceClient {
    pub(crate) id: String,
    pub(crate) socket: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
    pub(crate) user_id: Option<String>,
    pub(crate) request_ids: Arc<Mutex<Vec<String>>>,
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
    connection_loop().await;
}

async fn connection_loop() {
    let clients = Arc::new(Mutex::new(DashMap::new()));
    let mut incoming = SERVER.get().expect("Failed to get server").incoming();
    while let Some(stream) = incoming.next().await {
        let clients_arc = clients.clone();
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
            let client = VoiceClient {
                id: id.clone(),
                socket: socket_arc.clone(),
                user_id: None,
                request_ids: Arc::new(Mutex::new(request_ids.clone())),
            };
            clients_arc.lock().await.insert(id.clone(), client);
            let secret = EphemeralSecret::new(OsRng);
            let public_key = PublicKey::from(&secret);
            let mut buf = Vec::new();
            let val = RpcApiEvent {
                event: Event::Hello(HelloEvent {
                    public_key: public_key.to_bytes().to_vec(),
                    request_ids,
                }),
            };
            val.serialize(&mut Serializer::new(&mut buf).with_struct_map())
                .unwrap();
            // println!("test: {:?}", buf);
            let mut write = socket_arc.lock().await;
            write.send(Message::Binary(buf)).await.unwrap();
            drop(write);
            while let Some(data) = read.next().await {
                    match data.unwrap() {
                        Message::Binary(bin) => {
                            println!("Received binary data");
                            let mut deserializer = Deserializer::new(bin.as_slice());
                            let result: Result<RpcApiMethod, Error> =
                                Deserialize::deserialize(&mut deserializer);
                            if let Ok(r) = result {
                                println!("Received: {:?}", r.method);
                                if let Some(request_id) = r.id {
                                    let clients_locked = clients_arc.lock().await;
                                    let client = clients_locked.get(&id).unwrap();
                                    let mut request_ids = client.request_ids.lock().await;
                                    if request_ids.contains(&request_id) {
                                        request_ids.retain(|x| x != &request_id);
                                    } else {
                                        let error = Response::Error(ErrorResponse {
                                            error: "Invalid request id".to_string(),
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
                                    drop(clients_locked);
                                    let dispatch: Response = match r.method {
                                        Method::Identify(m) => {
                                            authentication::identify(
                                                m,
                                                clients_arc.clone(),
                                                id.clone(),
                                            )
                                            .await
                                        }
                                        Method::GetId(m) => {
                                            authentication::get_id(
                                                m,
                                                clients_arc.clone(),
                                                id.clone(),
                                            )
                                            .await
                                        }
                                        Method::Capabilities(m) => webrtc::capabilities(m).await,
                                        Method::Transport(m) => {
                                            webrtc::transport(m, clients_arc.clone(), id.clone())
                                                .await
                                        }
                                        Method::Dtls(m) => webrtc::dtls(m).await,
                                    Method::Produce(m) => webrtc::produce(m,
                                        clients_arc.clone(),
                                        id.clone(),).await,
                                    Method::Consume(m) => webrtc::consume(m,
                                        clients_arc.clone(),
                                        id.clone(),).await,
                                        Method::Resume(m) => webrtc::resume(m).await,
                                    Method::GetMessages(m) => messages::get_messages(m).await,
                                    Method::SendMessage(m) => messages::send_message(m, clients_arc.clone(), id.clone()).await,
                                    };
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
                                        error: "No request id".to_string(),
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
                                    error: "Invalid data or method not found".to_string(),
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
                            clients_arc.lock().await.remove(&id.clone());
                        drop(write);
                            break;
                        }
                        _ => {
                            println!("Received unknown message");
                        let mut write = socket_arc.lock().await;
                        write.close().await.unwrap();
                            clients_arc.lock().await.remove(&id.clone());
                        drop(write);
                            break;
                        }
                    }
                }
            
        });
    }
}
