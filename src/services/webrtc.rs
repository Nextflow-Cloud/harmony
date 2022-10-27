use std::num::{NonZeroU8, NonZeroU32};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
use std::sync::Arc;

use async_std::net::{IpAddr::V4, Ipv4Addr};
use async_std::sync::Mutex;
use dashmap::DashMap;
use lazy_static::lazy_static;
use mediasoup::consumer::{Consumer, ConsumerId, ConsumerOptions, ConsumerType};
use mediasoup::data_structures::{DtlsParameters, IceCandidate, IceParameters, ListenIp};
use mediasoup::producer::{Producer, ProducerId, ProducerOptions, ProducerType};
use mediasoup::router::{Router, RouterOptions};
use mediasoup::rtp_parameters::{
    MediaKind, RtpCapabilities, RtpCapabilitiesFinalized, RtpParameters, RtpCodecCapability, MimeTypeVideo, MimeTypeAudio,
};
use mediasoup::sctp_parameters::SctpParameters;
use mediasoup::transport::{ConsumeError, ProduceError, Transport, TransportId};
use mediasoup::webrtc_transport::{
    TransportListenIps, WebRtcTransport, WebRtcTransportOptions, WebRtcTransportRemoteParameters,
};
use mediasoup::worker::{RequestError, Worker, WorkerSettings, WorkerLogTag, WorkerLogLevel};
use mediasoup::worker_manager::WorkerManager;

use super::socket::VoiceClient;

lazy_static! {
    static ref WORKERS: Arc<Mutex<Vec<Arc<Worker>>>> = Arc::new(Mutex::new(Vec::new()));
    static ref WORKER_INDEX: AtomicUsize = AtomicUsize::new(0);
    static ref CALLS: DashMap<String, Arc<Mutex<Call>>> = DashMap::new();
}

pub struct CallMember {
    client: VoiceClient,
    transports: Arc<Mutex<Vec<String>>>,
    producers: Arc<Mutex<Vec<(String, MediaKind, RtpParameters)>>>,
    consumers: Arc<Mutex<Vec<String>>>,
}

pub struct Call {
    id: String,
    router: Router,
    transports: DashMap<String, WebRtcTransport>,
    producers: DashMap<String, Producer>,
    consumers: DashMap<String, Consumer>,
    members: DashMap<String, CallMember>,
}

impl Call {
    pub async fn new(id: String) -> Self {
        let worker = get_worker().await;
        let router_options = RouterOptions::new(vec![
            RtpCodecCapability::Audio {
                mime_type: MimeTypeAudio::Opus,
                clock_rate: NonZeroU32::new(48000).unwrap(),
                channels: NonZeroU8::new(2).unwrap(),
                parameters: Default::default(),
                rtcp_feedback: vec![],
                preferred_payload_type: None,
            }, RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::Vp8,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: Default::default(),
                rtcp_feedback: vec![],
                preferred_payload_type: None,
            }, RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::Vp9,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: Default::default(),
                rtcp_feedback: vec![],
                preferred_payload_type: None,
            }, RtpCodecCapability::Video {
                mime_type: MimeTypeVideo::H264,
                clock_rate: NonZeroU32::new(90000).unwrap(),
                parameters: Default::default(),
                rtcp_feedback: vec![],
                preferred_payload_type: None,
            }
        ]);
        let router = worker
            .create_router(router_options)
            .await
            .expect("Failed to create router");
        Call {
            id,
            router,
            transports: DashMap::new(),
            producers: DashMap::new(),
            consumers: DashMap::new(),
            members: DashMap::new(),
        }
    }
    pub fn get_rtp_capabilities(&self) -> RtpCapabilitiesFinalized {
        self.router.rtp_capabilities().clone()
    }
    pub async fn create_transport(
        &mut self,
        user: VoiceClient,
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
        if !(self.members)
            .iter()
            .any(|item| item.client.get_user_id() == user.get_user_id())
        {
            self.members.insert(user.get_user_id(), CallMember {
                client: user.clone(),
                transports: Arc::new(Mutex::new(Vec::new())),
                producers: Arc::new(Mutex::new(Vec::new())),
                consumers: Arc::new(Mutex::new(Vec::new())),
            });
        }
        let listen_ips = TransportListenIps::new(ListenIp {
            ip: V4(Ipv4Addr::new(0, 0, 0, 0)),
            announced_ip: Some(V4(Ipv4Addr::new(0, 0, 0, 0))), // TODO: use env instead of actual public ip
        });
        let transport = self
            .router
            .create_webrtc_transport(WebRtcTransportOptions::new(listen_ips))
            .await;
        match transport {
            Ok(t) => {
                let id = t.id();
                let ice_parameters = t.ice_parameters().clone();
                let ice_candidates = t.ice_candidates().clone();
                let dtls_parameters = t.dtls_parameters();
                let sctp_parameters = t.sctp_parameters();
                let member = self.members.get(&user.get_user_id()).unwrap();
                member.transports.lock().await.push(t.id().to_string());
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
    pub async fn connect_transport(&self, id: String, dtls_parameters: DtlsParameters) {
        let transport = self.transports.get(&id).unwrap();
        transport
            .connect(WebRtcTransportRemoteParameters { dtls_parameters })
            .await
            .unwrap();
    }
    pub async fn produce(
        &self,
        id: String,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
        user: VoiceClient,
    ) -> Result<(ProducerId, MediaKind, RtpParameters, ProducerType), ProduceError> {
        let transport = self.transports.get(&id).unwrap();
        let producer_options = ProducerOptions::new(kind, rtp_parameters);
        let producer = transport.produce(producer_options).await;
        match producer {
            Ok(p) => {
                let id = p.id();
                let kind = p.kind();
                let rtp_parameters = p.rtp_parameters().clone();
                let producer_type = p.r#type();
                let member = self.members.get(&user.get_user_id()).unwrap();
                member.producers.lock().await.push((p.id().to_string(), p.kind(), p.rtp_parameters().clone()));
                self.producers.insert(p.id().to_string(), p);
                Ok((id, kind, rtp_parameters, producer_type))
            }
            Err(e) => Err(e),
        }
    }
    pub async fn consume(
        &self,
        id: String,
        producer_id: ProducerId,
        rtp_capabilities: RtpCapabilities,
        user: VoiceClient,
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
                let id = c.id();
                let kind = c.kind();
                let rtp_parameters = c.rtp_parameters().clone();
                let consumer_type = c.r#type();
                let producer_id = c.producer_id();
                let producer_paused = c.producer_paused();
                let member = self.members.get(&user.get_user_id()).unwrap();
                member.consumers.lock().await.push(c.id().to_string());
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
    pub async fn resume(&self, consumer_id: String) {
        let consumer = self.consumers.get(&consumer_id).unwrap();
        consumer.resume().await.unwrap();
    }
    pub fn get_members(&self) -> Vec<(VoiceClient, Arc<Mutex<Vec<(String, MediaKind, RtpParameters)>>>)> {
        self.members.iter().map(move |item| {
            (item.client.clone(), item.producers.clone())
        }).collect()
    }
}

pub async fn create_workers() {
    let worker_manager = WorkerManager::new();
    let mut workers = WORKERS.lock().await;
    for _ in 0..num_cpus::get() {
        let mut worker_settings = WorkerSettings::default();
        worker_settings.rtc_ports_range = 10000..=11000;
        worker_settings.log_level = WorkerLogLevel::Debug;
        worker_settings.log_tags= vec![
            WorkerLogTag::Info,
            WorkerLogTag::Ice,
            WorkerLogTag::Dtls,
            WorkerLogTag::Rtp,
            WorkerLogTag::Srtp,
            WorkerLogTag::Rtcp,
            WorkerLogTag::Rtx,
            WorkerLogTag::Bwe,
            WorkerLogTag::Score,
            WorkerLogTag::Simulcast,
            WorkerLogTag::Svc,
            WorkerLogTag::Sctp,
            WorkerLogTag::Message,
        ];
        let worker = worker_manager
            .create_worker(worker_settings)
            .await
            .expect("Failed to create worker");
        workers.push(Arc::new(worker));
    }
}

pub async fn get_worker() -> Arc<Worker> {
    let index = WORKER_INDEX.load(Relaxed);
    let workers = WORKERS.lock().await;
    let worker = workers[index].clone();
    WORKER_INDEX.store((index + 1) % workers.len(), Relaxed);
    worker
}

pub async fn get_call(channel_id: String) -> Arc<Mutex<Call>> {
    let call = CALLS.get(&channel_id);
    match call {
        Some(c) => c.value().clone(),
        None => {
            let new_call = Call::new(channel_id.clone()).await;
            CALLS.insert(channel_id.clone(), Arc::new(Mutex::new(new_call)));
            CALLS.get(&channel_id).unwrap().value().clone()
        }
    }
}
