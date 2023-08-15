use async_std::task::{spawn, sleep};
use dashmap::DashMap;
use futures_util::StreamExt;
use jsonwebtoken::{encode, Header, EncodingKey};
use lazy_static::lazy_static;
use redis::{AsyncCommands, FromRedisValue, ToRedisArgs};
use serde::{Deserialize, Serialize};

use crate::errors::{Result, Error};

use super::{redis::get_connection, socket::{deserialize, serialize}, environment::JWT_SECRET, database::calls::Call};

lazy_static! {
    pub static ref AVAILABLE_NODES: DashMap<String, Node> = DashMap::new();
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeDescription {
    id: String,
    region: Region,
}

pub struct Node {
    id: String,
    region: Region,
    last_ping: i64,
}

impl Node {
    pub fn suppress(&self) {
        // TODO: disable node and clean up calls (move to other server if possible)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeEvent {
    id: String,
    #[serde(flatten)]
    event: NodeEventKind,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum NodeEventKind {
    Description(NodeDescription),
    Ping,
    Disconnect,
    Timeout(CallUser),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ServerEventKind {

}

impl From<NodeDescription> for Node {
    fn from(node: NodeDescription) -> Self {
        let time = chrono::Utc::now().timestamp_millis();
        Node {
            id: node.id,
            region: node.region,
            last_ping: time,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Region {
    Canada,
    UsCentral,
    UsEast,
    UsWest,
    Europe,
    Asia,
    SouthAmerica,
    Australia,
    Africa,
}

pub fn spawn_check_available_nodes() {
    spawn(async move {
        loop {
            let redis = get_connection().await;
            let mut pubsub = redis.into_pubsub();
            pubsub.subscribe("nodes").await.unwrap();
            while let Some(msg) = pubsub.on_message().next().await {
                let payload: NodeEvent = msg.get_payload().unwrap();
                match payload {
                    NodeEvent { event: NodeEventKind::Description(description), .. } => {
                        let node: Node = description.into();
                        AVAILABLE_NODES.insert(node.id.clone(), node);
                    }
                    NodeEvent { id, event: NodeEventKind::Ping } => {
                        let mut node = AVAILABLE_NODES.get_mut(&id).unwrap();
                        node.last_ping = chrono::Utc::now().timestamp_millis();
                    }
                    NodeEvent { id, event: NodeEventKind::Disconnect } => {
                        AVAILABLE_NODES.remove(&id);
                    }
                    NodeEvent { event: NodeEventKind::Timeout(user), .. } => {
                        // clean up after user
                        let call = ActiveCall::get(&user.call_id).await.unwrap();
                        if call.is_none() {
                            continue;
                        }
                        let mut call = call.unwrap();
                        call.leave_user(&user.id).await.expect("Failed to leave user");
                    }
                }

            }
        }
    });
    spawn(async move {
        loop {
            let time = chrono::Utc::now().timestamp_millis();
            let mut nodes = AVAILABLE_NODES.iter_mut();
            while let Some(node) = nodes.next() {
                if node.value().last_ping + 10000 < time {
                    node.value().suppress();
                    // Remove node
                    let id = node.key().clone();
                    drop(node);
                    AVAILABLE_NODES.remove(&id);
                }
            }
            // Don't deadlock
            sleep(std::time::Duration::from_millis(1000)).await;
        }
    });
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActiveCall {
    pub id: String,
    pub name: Option<String>,
    pub members: Vec<String>,
    pub space_id: String,
    pub channel_id: String, 
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CallUser {
    id: String,
    call_id: String,
    muted: bool,
    deafened: bool,
    speaking: bool,
    video: bool,
    screenshare: bool,
}

impl FromRedisValue for ActiveCall {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match *v {
            redis::Value::Data(ref bytes) => {
                let data = deserialize(bytes);
                match data {
                    Ok (data) => Ok(data),
                    Err(_) => {
                        return Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization error")));
                    } 
                }
            }


            _ => Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Format error"
            ))),
        }
    }
}

impl FromRedisValue for NodeEvent {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match *v {
            redis::Value::Data(ref bytes) => {
                let data = deserialize(bytes);
                match data {
                    Ok (data) => Ok(data),
                    Err(_) => {
                        return Err(redis::RedisError::from((redis::ErrorKind::TypeError, "Deserialization error")));
                    } 
                }
            }


            _ => Err(redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Format error"
            ))),
        }
    }
}

impl ToRedisArgs for ActiveCall {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite {
            let data = serialize(self).unwrap();
            out.write_arg(data.as_slice());
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RtcAuthorization {
    call_id: String,
    user_id: String,
}



impl ActiveCall {
    pub async fn create(space: &String, channel: &String, initiator: &String) -> Result<ActiveCall> {
        let mut redis = get_connection().await;
        let call = Self::get_in_channel(space, channel).await?;
        if call.is_some() {
            return Err(Error::AlreadyExists);
        }
        let call = ActiveCall {
            id: ulid::Ulid::new().to_string(),
            name: None,
            members: vec![initiator.clone()],
            space_id: space.clone(),
            channel_id: channel.clone(),

        };
        redis.set::<std::string::String, ActiveCall, ActiveCall>(format!("call:{}:{}", space, channel), call.clone()).await.unwrap();
        let stored_call = Call {
            channel_id: channel.clone(),
            id: call.id.clone(),
            joined_members: vec![initiator.clone()],
            name: None,
            ended_at: chrono::Utc::now().timestamp_millis(),
        };
        stored_call.create().await?;
        let space = space.clone();
        let channel = channel.clone();
        spawn(async move {
            loop {
                sleep(std::time::Duration::from_millis(30000)).await;
                let mut redis = get_connection().await;
                let active_call: std::result::Result<Option<ActiveCall>, _> = redis.get(format!("call:{}:{}", space, channel)).await;
                let active_call = match active_call {
                    Ok(call) => call,
                    Err(_) => {
                        break;
                    }
                };
                if active_call.is_none() {
                    break;
                }
                let active_call = active_call.unwrap();
                Call::update(&active_call.id, active_call.members.clone()).await.unwrap(); // FIXME: unwrap
            }
        });
        Ok(call)
    }

    pub async fn get_in_channel(space: &String, channel: &String) -> Result<Option<ActiveCall>> {
        let mut redis = get_connection().await;
        let id: Option<String> = redis.get(format!("call:{}:{}", space, channel)).await?;
        if let Some(id) = id {
            Ok(Self::get(&id).await?)
        } else {
            Ok(None)
        }
    }

    pub async fn get(id: &String) -> Result<Option<ActiveCall>> {
        let mut redis = get_connection().await;
        let call: Option<ActiveCall> = redis.get(format!("call:{}", id)).await?;
        Ok(call)
    }

    pub async fn update(&self) -> Result<()> {
        let mut redis = get_connection().await;
        redis.set::<String, ActiveCall, ActiveCall>(format!("call:{}", self.id), self.clone()).await?;
        Ok(())
    }

    pub async fn join_user(&mut self, id: String) -> Result<()> { // add Result<()>?
        self.members.push(id);
        self.update().await?;
        Ok(())
    }

    pub async fn get_token(&self, user_id: &String) -> Result<String> {
        let authorization = RtcAuthorization {
            user_id: user_id.to_string(),
            call_id: self.id.clone(),
        };
        let token = encode::<RtcAuthorization>(&Header::default(), &authorization, &EncodingKey::from_secret(JWT_SECRET.as_bytes()))?;
        Ok(token)
    }

    pub async fn leave_user(&mut self, user_id: &String) -> Result<()> {
        // remove user from call
        self.members.retain(|x| x != user_id);
        self.update().await?;
        // then end the call if there are no users present
        if self.members.len() == 0 {
            self.end().await?;
        }
        Ok(())
    }

    pub async fn end(&self) -> Result<()> {
        // remove call from redis, store into db
        let mut redis = get_connection().await;
        redis.del::<std::string::String, ActiveCall>(format!("call:{}:{}", self.space_id, self.channel_id)).await?;
        
        // disconnect any remaining users present
        Ok(())
    }
}

