use std::sync::Arc;

use async_std::{
    channel::{unbounded, Sender},
    future,
    net::{TcpListener, TcpStream},
    task::spawn,
};
use async_tungstenite::{accept_async, tungstenite::Message};
use dashmap::DashMap;
use futures_util::{SinkExt, StreamExt};
use rand::rngs::OsRng;
use rmp_serde::{Deserializer, Serializer};
use serde::{Deserialize, Serialize};
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::{
    errors::Error,
    globals::HEARTBEAT_TIMEOUT,
    methods::{get_respond, Event, HelloEvent, RpcApiEvent, RpcApiMethod, RpcApiResponse},
    services::encryption::generate_id,
};

use super::{database::users::User, environment::LISTEN_ADDRESS};

#[derive(Clone)]
pub struct RpcClient {
    pub id: String,
    pub socket: Arc<Sender<Message>>,
    pub user: Option<Arc<User>>,
    pub request_ids: Vec<String>,
    pub heartbeat_tx: Arc<Sender<()>>,
}

pub async fn start_server() {
    let server = TcpListener::bind(LISTEN_ADDRESS.to_owned()).await.unwrap();
    let clients: Arc<DashMap<String, RpcClient>> = Arc::new(DashMap::new());
    let mut incoming = server.incoming();
    while let Some(stream) = incoming.next().await {
        let clients = clients.clone();
        spawn(async move { start_client(stream, clients).await });
    }
}

async fn start_client(
    stream: Result<TcpStream, std::io::Error>,
    clients: Arc<DashMap<String, RpcClient>>,
) {
    let connection = stream.unwrap();
    println!("Socket connected: {}", connection.peer_addr().unwrap());
    let ws_stream = accept_async(connection).await.expect("Failed to accept");
    let (mut write, mut read) = ws_stream.split();
    let (s, r) = unbounded::<Message>();
    spawn(async move {
        while let Ok(msg) = r.recv().await {
            write.send(msg).await.expect("Failed to send message");
        }
        write.close().await.expect("Failed to close");
    });
    let id = generate_id();
    let mut request_ids = Vec::new();
    for _ in 0..20 {
        request_ids.push(generate_id());
    }
    let secret = EphemeralSecret::random_from_rng(OsRng);
    let public_key = PublicKey::from(&secret);
    let val = RpcApiEvent {
        event: Event::Hello(HelloEvent {
            public_key: public_key.to_bytes().to_vec(),
            request_ids: request_ids.clone(),
        }),
    };
    s.send(Message::Binary(serialize(&val))).await.unwrap();

    let (tx, rx) = unbounded::<()>();
    let clients_moved = clients.clone();
    let id_moved = id.clone();
    spawn(async move {
        while future::timeout(
            std::time::Duration::from_millis(*HEARTBEAT_TIMEOUT),
            rx.recv(),
        )
        .await
        .is_ok()
        {}
        if let Some((_, client)) = clients_moved.remove(&id_moved) {
            client.socket.close();
        }
    });
    let client = RpcClient {
        id: id.clone(),
        socket: Arc::new(s),
        user: None,
        request_ids,
        heartbeat_tx: Arc::new(tx),
    };
    clients.insert(id.clone(), client);
    while let Some(data) = read.next().await {
        match data.unwrap() {
            Message::Binary(bin) => {
                println!("Received binary data");
                let response = handle_packet(bin, &clients, &id).await;
                let client = clients.get(&id.clone()).unwrap();
                client
                    .socket
                    .send(Message::Binary(serialize(&response)))
                    .await
                    .unwrap();
            }
            Message::Ping(bin) => {
                println!("Received ping");
                let client = clients.get(&id.clone()).unwrap();
                client.socket.send(Message::Pong(bin)).await.unwrap();
            }
            Message::Close(_) => {
                println!("Received close");
            }
            _ => {
                println!("Received unknown message");
                if let Some((_, client)) = clients.remove(&id.clone()) {
                    client.socket.close();
                }
            }
        }
    }
}

pub async fn handle_packet(
    bin: Vec<u8>,
    clients: &Arc<DashMap<String, RpcClient>>,
    id: &String,
) -> RpcApiResponse {
    let result = deserialize::<RpcApiMethod>(bin.as_slice());
    if let Ok(r) = result {
        println!("Received: {:?}", r.method);
        if let Some(request_id) = r.id {
            let mut client = clients.get_mut(id).unwrap();
            if client.request_ids.contains(&request_id) {
                client.request_ids.retain(|x| x != &request_id);
            } else {
                return RpcApiResponse {
                    id: None,
                    response: None,
                    error: Some(Error::InvalidRequestId),
                };
            }
            drop(client);
            let dispatch = get_respond(r.method)
                .respond(clients.clone(), id.clone())
                .await;
            if let Ok(dispatch) = dispatch {
                RpcApiResponse {
                    id: Some(request_id),
                    response: Some(dispatch),
                    error: None,
                }
            } else {
                RpcApiResponse {
                    id: Some(request_id),
                    response: None,
                    error: Some(dispatch.unwrap_err()),
                }
            }
        } else {
            RpcApiResponse {
                id: None,
                response: None,
                error: Some(Error::InvalidRequestId),
            }
        }
    } else {
        RpcApiResponse {
            id: None,
            response: None,
            error: Some(Error::InvalidMethod),
        }
    }
}

pub fn serialize<T: Serialize>(value: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    value
        .serialize(&mut Serializer::new(&mut buf).with_struct_map())
        .unwrap();
    buf
}

pub fn deserialize<T: for<'a> Deserialize<'a>>(buf: &[u8]) -> Result<T, rmp_serde::decode::Error> {
    let mut deserializer = Deserializer::new(buf);
    Deserialize::deserialize(&mut deserializer)
}
