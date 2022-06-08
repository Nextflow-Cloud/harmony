use std::sync::Arc;

use async_std::net::TcpStream;
use async_std::net::{IpAddr::V4, Ipv4Addr};
use async_std::sync::Mutex;
use async_tungstenite::WebSocketStream;
use mediasoup::prelude::{TransportListenIp, TransportListenIps};

use crate::methods::CapabilitiesMethod;
use crate::services::webrtc;
use crate::Response;

use super::{
    CapabilitiesResponse, ConsumeMethod, ConsumeResponse, DtlsMethod, DtlsResponse,
    NotFoundResponse, ProduceMethod, ProduceResponse, ResumeMethod, ResumeResponse,
    TransportMethod, TransportResponse,
};

pub async fn capabilities(
    socket: Arc<Mutex<WebSocketStream<TcpStream>>>,
    method: CapabilitiesMethod,
) -> Response {
    // let channel_id = "".to_string();
    // let router = webrtc::create_router(method.id).await;
    // let call = webrtc::Call::new(method.id).await;
    // let capabilities = router.lock().await.rtp_capabilities();
    let call = webrtc::get_call(method.channel_id).await;
    Response::Capabilities(CapabilitiesResponse {
        rtp_capabilities: call.get_rtp_capabilities(),
    })
}

pub async fn transport(method: TransportMethod) -> Response {
    // let router = webrtc::create_router(method.id).await;
    let call = webrtc::get_call(method.channel_id).await;
    let listen_ips = TransportListenIps::new(TransportListenIp {
        ip: V4(Ipv4Addr::new(0, 0, 0, 0)),
        announced_ip: Some(V4(Ipv4Addr::new(24, 141, 115, 80))),
    });
    // let transport = router.lock()
    //     .await
    //     .create_webrtc_transport(WebRtcTransportOptions::new(listen_ips))
    //     .await
    //     .expect("Uh oh");
    let transport = call.create_transport().await;
    match transport {
        Ok(t) => Response::Transport(TransportResponse {
            id: t.0,
            ice_parameters: t.1,
            ice_candidates: t.2,
            dtls_parameters: t.3,
            sctp_parameters: t.4,
        }),
        Err(e) => Response::NotFound(NotFoundResponse {
            error: "Failed to create transport.".to_string(),
        }),
    }
}

pub async fn dtls(method: DtlsMethod) -> Response {
    let call = webrtc::get_call(method.channel_id).await; // very messy code, didn't test
    call.connect_transport(method.transport_id, method.dtls_parameters)
        .await;
    Response::Dtls(DtlsResponse {})
}

pub async fn produce(method: ProduceMethod) -> Response {
    let call = webrtc::get_call(method.channel_id).await;
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
        Err(e) => {
            Response::NotFound(NotFoundResponse {
                error: "An error occurred while attempting to produce.".to_string(),
            }) // TODO: Don't use NotFound, use error property
        }
    }
}

pub async fn consume(method: ConsumeMethod) -> Response {
    let call = webrtc::get_call(method.channel_id).await;
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
        Err(e) => {
            Response::NotFound(NotFoundResponse {
                error: "An error occurred while attempting to consume.".to_string(),
            }) // TODO: Don't use NotFound, use error property
        }
    }
}

pub async fn resume(method: ResumeMethod) -> Response {
    let call = webrtc::get_call(method.channel_id).await;
    call.resume(method.consumer_id).await;
    Response::Resume(ResumeResponse {})
}
