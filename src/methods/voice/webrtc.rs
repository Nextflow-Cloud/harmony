use std::sync::Arc;

use async_std::sync::Mutex;
use dashmap::DashMap;

use crate::methods::voice::CapabilitiesMethod;
use crate::methods::voice::Response;
use crate::services::socket::VoiceClient;
use crate::services::webrtc;

use super::{
    CapabilitiesResponse, ConsumeMethod, ConsumeResponse, DtlsMethod, DtlsResponse, ErrorResponse,
    ProduceMethod, ProduceResponse, ResumeMethod, ResumeResponse, TransportMethod,
    TransportResponse,
};

pub async fn capabilities(method: CapabilitiesMethod) -> Response {
    let call_mutex = webrtc::get_call(method.channel_id).await;
    let call = call_mutex.lock().await;
    Response::Capabilities(CapabilitiesResponse {
        rtp_capabilities: call.get_rtp_capabilities(),
    })
}

pub async fn transport(
    method: TransportMethod,
    clients: Arc<Mutex<DashMap<String, VoiceClient>>>,
    id: String,
) -> Response {
    let clients_locked = clients.lock().await;
    let client = clients_locked.get(&id).unwrap();
    let call_mutex = webrtc::get_call(method.channel_id).await;
    let mut call = call_mutex.lock().await;
    let transport = call.create_transport(client.clone()).await;
    match transport {
        Ok(t) => Response::Transport(TransportResponse {
            id: t.0,
            ice_parameters: t.1,
            ice_candidates: t.2,
            dtls_parameters: t.3,
            sctp_parameters: t.4,
        }),
        Err(_) => Response::Error(ErrorResponse {
            error: "Failed to create transport.".to_string(),
        }), // Uh oh
            // TODO: catch and log all errors using Logger
    }
}

pub async fn dtls(method: DtlsMethod) -> Response {
    let call_mutex = webrtc::get_call(method.channel_id).await;
    let call = call_mutex.lock().await;
    call.connect_transport(method.transport_id, method.dtls_parameters)
        .await;
    Response::Dtls(DtlsResponse {})
}

pub async fn produce(method: ProduceMethod) -> Response {
    let call_mutex = webrtc::get_call(method.channel_id).await;
    let call = call_mutex.lock().await;
    let produce = call
        .produce(method.transport_id, method.kind, method.rtp_parameters)
        .await;
    match produce {
        Ok(p) => Response::Produce(ProduceResponse {
            id: p.0,
            kind: p.1,
            rtp_parameters: p.2,
            producer_type: p.3,
        }),
        Err(_) => Response::Error(ErrorResponse {
            error: "An error occurred while attempting to produce.".to_string(),
        }),
    }
}

pub async fn consume(method: ConsumeMethod) -> Response {
    let call_mutex = webrtc::get_call(method.channel_id).await;
    let call = call_mutex.lock().await;
    let consume = call
        .consume(
            method.transport_id,
            method.producer_id,
            method.rtp_capabilities,
        )
        .await;
    match consume {
        Ok(c) => Response::Consume(ConsumeResponse {
            id: c.0,
            kind: c.1,
            rtp_parameters: c.2,
            consumer_type: c.3,
            producer_id: c.4,
            producer_paused: c.5,
        }),
        Err(_) => Response::Error(ErrorResponse {
            error: "An error occurred while attempting to consume.".to_string(),
        }),
    }
}

pub async fn resume(method: ResumeMethod) -> Response {
    let call_mutex = webrtc::get_call(method.channel_id).await;
    let call = call_mutex.lock().await;
    call.resume(method.consumer_id).await;
    Response::Resume(ResumeResponse {})
}
