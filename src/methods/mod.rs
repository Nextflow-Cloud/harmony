use mediasoup::{
    consumer::{ConsumerId, ConsumerType},
    prelude::{DtlsParameters, IceCandidate, IceParameters},
    producer::{ProducerId, ProducerType},
    rtp_parameters::{MediaKind, RtpCapabilities, RtpCapabilitiesFinalized, RtpParameters},
    sctp_parameters::SctpParameters,
    transport::TransportId,
};
use serde::{Deserialize, Serialize};

use crate::services::database::messages::Message;

pub mod authentication;
pub mod webrtc;
pub mod messages;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(i8)]
pub enum Method {
    Identify(IdentifyMethod) = 1,
    // Heartbeat(HeartbeatMethod) = 2,
    GetId(GetIdMethod) = 5,
    Capabilities(CapabilitiesMethod) = 10,
    Transport(TransportMethod) = 11,
    Dtls(DtlsMethod) = 12,
    Produce(ProduceMethod) = 13,
    Consume(ConsumeMethod) = 14,
    Resume(ResumeMethod) = 15,

    GetChannelMessages(GetChannelMessagesMethod) = 20,
    SendChannelMessage(SendChannelMessageMethod) = 22,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VoiceApiMethod {
    pub(crate) id: Option<String>,
    #[serde(flatten)]
    pub(crate) method: Method,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentifyMethod {
    public_key: Vec<u8>,
    token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetIdMethod {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CapabilitiesMethod {
    channel_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransportMethod {
    channel_id: String,
    producer: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DtlsMethod {
    channel_id: String,
    transport_id: String,
    dtls_parameters: DtlsParameters,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProduceMethod {
    channel_id: String,
    transport_id: String,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsumeMethod {
    channel_id: String,
    transport_id: String,
    producer_id: ProducerId,
    rtp_capabilities: RtpCapabilities,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResumeMethod {
    channel_id: String,
    consumer_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetChannelMessagesMethod {
    channel_id: String,
    limit: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendChannelMessageMethod {
    channel_id: String,
    content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(i8)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Response {
    Identify(IdentifyResponse) = 1,

    Error(ErrorResponse) = 4,
    GetId(GetIdResponse) = 5,

    Capabilities(CapabilitiesResponse) = 10,
    Transport(TransportResponse) = 11,
    Dtls(DtlsResponse) = 12,
    Produce(ProduceResponse) = 13,
    Consume(ConsumeResponse) = 14,
    Resume(ResumeResponse) = 15,

    GetChannelMessages(GetChannelMessagesResponse) = 20,
    SendChannelMessage(SendChannelMessageResponse) = 22,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceApiResponse {
    pub(crate) id: Option<String>,
    #[serde(flatten)]
    pub(crate) response: Response,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ErrorResponse {
    pub(crate) error: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetIdResponse {
    pub(crate) request_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IdentifyResponse {
    success: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CapabilitiesResponse {
    rtp_capabilities: RtpCapabilitiesFinalized,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TransportResponse {
    id: TransportId,
    ice_parameters: IceParameters,
    ice_candidates: Vec<IceCandidate>,
    dtls_parameters: DtlsParameters,
    sctp_parameters: Option<SctpParameters>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DtlsResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProduceResponse {
    id: ProducerId,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
    producer_type: ProducerType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ConsumeResponse {
    id: ConsumerId,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
    consumer_type: ConsumerType,
    producer_id: ProducerId,
    producer_paused: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResumeResponse {}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GetChannelMessagesResponse {
    messages: Vec<Message>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SendChannelMessageResponse {
    message_id: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[repr(i8)]
#[serde(tag = "type", content = "data", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Event {
    Hello(HelloEvent) = 0,

    NewProducer(NewProducerEvent) = 16,

    NewChannelMessage(NewChannelMessageEvent) = 21,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct VoiceApiEvent {
    #[serde(flatten)]
    pub(crate) event: Event,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HelloEvent {
    pub(crate) public_key: Vec<u8>,
    pub(crate) request_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewProducerEvent {
    id: ProducerId,
    kind: MediaKind,
    rtp_parameters: RtpParameters,
    producer_type: ProducerType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewChannelMessageEvent {
    message: Message,
}
