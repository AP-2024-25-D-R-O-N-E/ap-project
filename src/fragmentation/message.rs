use std::mem;

use serde::{Deserialize, Serialize};
use wg_2024::network::NodeId;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Message {
    origin_id: NodeId,
    destination_id: NodeId,
    pub message_data: MessageData,
}

impl Message {
    pub fn new(origin_id: NodeId, destination_id: NodeId, message_data: MessageData) -> Self {
        Self {
            origin_id,
            destination_id,
            message_data,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageData {
    RegisterAsClient(NodeId), // for now lets just use NodeId, maybe in the future we allow for different ids
    RequestClients,
    TextMessage{from: NodeId, to: NodeId, text: String},

    ResponseClients(Vec<NodeId>),
    AcknolewdgedAsClient,
}

impl Message {
    pub fn into_u8(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_u8(v: Vec<u8>) -> Self {
        bincode::deserialize(&v).unwrap()
    }
}
