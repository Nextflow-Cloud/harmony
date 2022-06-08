use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
// use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::Ordering::Relaxed;

use async_std::net::IpAddr::V4;
use async_std::net::Ipv4Addr;
use async_std::net::TcpStream;
use async_std::sync::Mutex;
use async_tungstenite::WebSocketStream;
use dashmap::DashMap;
use lazy_static::lazy_static;
use mediasoup::consumer::{Consumer, ConsumerId, ConsumerOptions, ConsumerType};
use mediasoup::prelude::DtlsParameters;
use mediasoup::prelude::IceCandidate;
use mediasoup::prelude::IceParameters;
use mediasoup::prelude::TransportListenIp;
use mediasoup::producer::ProducerType;
use mediasoup::producer::{Producer, ProducerId, ProducerOptions};
use mediasoup::router::{Router, RouterOptions};
use mediasoup::rtp_parameters::RtpCapabilities;
use mediasoup::rtp_parameters::RtpCapabilitiesFinalized;
use mediasoup::rtp_parameters::{MediaKind, RtpParameters};
use mediasoup::sctp_parameters::SctpParameters;
use mediasoup::transport::ConsumeError;
use mediasoup::transport::ProduceError;
use mediasoup::transport::Transport;
use mediasoup::transport::TransportId;
use mediasoup::webrtc_transport::{
    TransportListenIps, WebRtcTransport, WebRtcTransportOptions, WebRtcTransportRemoteParameters,
};
use mediasoup::worker::RequestError;
use mediasoup::worker::{Worker, WorkerSettings};
use mediasoup::worker_manager::WorkerManager;

lazy_static! {
    static ref WORKERS: Arc<Mutex<Vec<Arc<Mutex<Worker>>>>> = Arc::new(Mutex::new(Vec::new()));
    // static ref ROUTERS: DashMap<String, Arc<Mutex<Router>>> = DashMap::new();
    static ref WORKER_INDEX: AtomicUsize = AtomicUsize::new(0);
    static ref CALLS: DashMap<String, Arc<Mutex<Call>>> = DashMap::new();
}

pub struct CallMember {
    id: String,
    user_id: String,
    socket: Arc<Mutex<WebSocketStream<TcpStream>>>,
}

pub struct Call {
    id: String,
    router: Router,
    transports: DashMap<String, WebRtcTransport>,
    producers: DashMap<String, Producer>,
    consumers: DashMap<String, Consumer>,
    members: Vec<CallMember>,
}

impl Call {
    pub async fn new(id: String) -> Self {
        let worker_arc = get_worker().await;
        let worker = worker_arc.lock().await;
        let router = worker
            .create_router(RouterOptions::default())
            .await
            .expect("Failed to create router");
        Call {
            id,
            router,
            transports: DashMap::new(),
            producers: DashMap::new(),
            consumers: DashMap::new(),
            members: Vec::new(),
        }
    }
    pub fn get_rtp_capabilities(self: &Self) -> RtpCapabilitiesFinalized {
        self.router.rtp_capabilities().clone()
    }
    pub async fn create_transport(
        self: &Self,
    ) -> Result<
        (
            TransportId,
            IceParameters,
            Vec<IceCandidate>,
            DtlsParameters,
            Option<SctpParameters>,
        ),
        RequestError,
    > {
        let listen_ips = TransportListenIps::new(TransportListenIp {
            ip: V4(Ipv4Addr::new(0, 0, 0, 0)),
            announced_ip: Some(V4(Ipv4Addr::new(0, 0, 0, 0))), // TODO: use env instead of actual public ip
        });
        let transport = self
            .router
            .create_webrtc_transport(WebRtcTransportOptions::new(listen_ips))
            .await;
        match transport {
            Ok(t) => {
                let id = t.id().clone();
                let ice_parameters = t.ice_parameters().clone();
                let ice_candidates = t.ice_candidates().clone();
                let dtls_parameters = t.dtls_parameters().clone();
                let sctp_parameters = t.sctp_parameters().clone();
                self.transports.insert(t.id().to_string(), t);
                Ok((
                    id,
                    ice_parameters,
                    ice_candidates,
                    dtls_parameters,
                    sctp_parameters,
                ))
            }
            Err(e) => Err(e),
        }
    }
    pub async fn connect_transport(self: &Self, id: String, dtls_parameters: DtlsParameters) {
        let transport = self.transports.get(&id).unwrap();
        transport
            .connect(WebRtcTransportRemoteParameters { dtls_parameters })
            .await
            .unwrap();
    }
    pub async fn produce(
        self: &Self,
        id: String,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
    ) -> Result<(ProducerId, MediaKind, RtpParameters, ProducerType), ProduceError> {
        let transport = self.transports.get(&id).unwrap();
        let producer_options = ProducerOptions::new(kind, rtp_parameters);
        let producer = transport.produce(producer_options).await;
        match producer {
            Ok(p) => {
                let id = p.id().clone();
                let kind = p.kind().clone();
                let rtp_parameters = p.rtp_parameters().clone();
                let producer_type = p.r#type().clone();
                self.producers.insert(p.id().to_string(), p);
                Ok((id, kind, rtp_parameters, producer_type))
            }
            Err(e) => Err(e),
        }
    }
    pub async fn consume(
        self: &Self,
        id: String,
        producer_id: ProducerId,
        rtp_capabilities: RtpCapabilities,
    ) -> Result<
        (
            ConsumerId,
            MediaKind,
            RtpParameters,
            ConsumerType,
            ProducerId,
            bool,
        ),
        ConsumeError,
    > {
        let transport = self.transports.get(&id).unwrap();
        let mut consumer_options = ConsumerOptions::new(producer_id, rtp_capabilities);
        consumer_options.paused = true;
        let consumer = transport.consume(consumer_options).await;
        match consumer {
            Ok(c) => {
                let id = c.id().clone();
                let kind = c.kind().clone();
                let rtp_parameters = c.rtp_parameters().clone();
                let consumer_type = c.r#type().clone();
                let producer_id = c.producer_id().clone();
                let producer_paused = c.producer_paused().clone();
                self.consumers.insert(c.id().to_string(), c);
                Ok((
                    id,
                    kind,
                    rtp_parameters,
                    consumer_type,
                    producer_id,
                    producer_paused,
                ))
            }
            Err(e) => Err(e),
        }
    }
    pub async fn resume(self: &Self, consumer_id: String) {
        let consumer = self.consumers.get(&consumer_id).unwrap();
        consumer.resume().await.unwrap();
    }
}

// pub async fn get_routers() -> DashMap<String, Arc<Mutex<Router>>> {
//     ROUTERS.clone()
// }

pub async fn create_workers() -> () {
    let worker_manager = WorkerManager::new();
    let mut workers = WORKERS.lock().await;
    for _ in 0..num_cpus::get() {
        let worker = worker_manager
            .create_worker(WorkerSettings::default())
            .await
            .expect("Failed to create worker");
        // worker.create_router(RouterOptions::default())
        //     .await
        //     .expect("Failed to create router");
        workers.push(Arc::new(Mutex::new(worker)));
    }
}

pub async fn get_worker() -> Arc<Mutex<Worker>> {
    let index = WORKER_INDEX.load(Relaxed);
    let workers = WORKERS.lock().await;
    let worker = workers[index].clone();
    WORKER_INDEX.store((index + 1) % workers.len(), Relaxed);
    worker
}

// pub async fn create_router(channel_id: String) -> Arc<Mutex<Router>> {
//     let routers = get_routers().await;
//     let router = routers.get(&channel_id);
//     if router.is_none() {
//         let worker_arc = get_worker().await;
//         let worker = worker_arc.lock().await;
//         let new_router = worker.create_router(RouterOptions::default())
//             .await
//             .expect("Failed to create router");
//         let router_arc = Arc::new(Mutex::new(new_router));
//         routers.insert(channel_id, router_arc.clone());
//         router_arc
//     } else {
//         router.unwrap().clone()
//     }
// }

pub async fn get_call(channel_id: String) -> Arc<Mutex<Call>> {
    let call = CALLS.get(&channel_id);
    match call {
        Some(c) => c.value().clone(),
        None => {
            let new_call = Call::new(channel_id.clone()).await;
            CALLS
                .insert(channel_id.clone(), Arc::new(Mutex::new(new_call)))
                .unwrap()
        }
    }
}
