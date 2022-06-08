use mediasoup::{
    consumer::{ConsumerId, ConsumerType},
    prelude::{DtlsParameters, IceCandidate, IceParameters},
    producer::{ProducerId, ProducerType},
    rtp_parameters::{MediaKind, RtpCapabilities, RtpCapabilitiesFinalized, RtpParameters},
    sctp_parameters::SctpParameters,
    transport::TransportId,
};
use serde::{Deserialize, Serialize};

pub mod authentication;
pub mod messages;
pub mod webrtc;

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[repr(i8)]
pub enum Method {
    Identify(IdentifyMethod) = 1,
    // Heartbeat(HeartbeatMethod) = 2,
    Capabilities(CapabilitiesMethod) = 10,
    Transport(TransportMethod) = 11,
    Dtls(DtlsMethod) = 12,
    Produce(ProduceMethod) = 13,
    Consume(ConsumeMethod) = 14,
    Resume(ResumeMethod) = 15,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct RpcApiMethod {
    pub(crate) id: Option<String>,
    pub(crate) data: Option<Method>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct IdentifyMethod {
    public_key: Vec<u8>,
    token: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CapabilitiesMethod {
    channel_id: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransportMethod {
    channel_id: String,
    producer: bool,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DtlsMethod {
    channel_id: String,
    transport_id: String,
    dtls_parameters: DtlsParameters,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ProduceMethod {
    channel_id: String,
    transport_id: String,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ConsumeMethod {
    channel_id: String,
    transport_id: String,
    producer_id: ProducerId,
    rtp_capabilities: RtpCapabilities,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ResumeMethod {
    channel_id: String,
    consumer_id: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[repr(i8)]
pub enum Response {
    Identify(IdentifyResponse) = 1,

    NotFound(NotFoundResponse) = 3,

    Capabilities(CapabilitiesResponse) = 10,
    Transport(TransportResponse) = 11,
    Dtls(DtlsResponse) = 12,
    Produce(ProduceResponse) = 13,
    Consume(ConsumeResponse) = 14,
    Resume(ResumeResponse) = 15,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct RpcApiResponse {
    pub(crate) id: Option<String>,
    pub(crate) error: Option<String>,
    pub(crate) data: Option<Response>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct NotFoundResponse {
    pub(crate) error: String,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct IdentifyResponse {
    success: bool,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct CapabilitiesResponse {
    rtp_capabilities: RtpCapabilitiesFinalized,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct TransportResponse {
    id: TransportId,
    ice_parameters: IceParameters,
    ice_candidates: Vec<IceCandidate>,
    dtls_parameters: DtlsParameters,
    sctp_parameters: Option<SctpParameters>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct DtlsResponse {}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ProduceResponse {
    id: ProducerId,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
    producer_type: ProducerType,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ConsumeResponse {
    id: ConsumerId,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
    consumer_type: ConsumerType,
    producer_id: ProducerId,
    producer_paused: bool,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct ResumeResponse {}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[repr(i8)]
pub enum Event {
    Hello(HelloEvent) = 0,

    NewProducer(NewProducerEvent) = 16,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct RpcApiEvent {
    pub(crate) data: Option<Event>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct HelloEvent {
    pub(crate) public_key: Vec<u8>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct NewProducerEvent {
    id: ProducerId,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
    producer_type: ProducerType,
}
