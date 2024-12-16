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
    RegisterAsClient,
    RequestClients,
    TextMessage(String),

    ResponseClients(Vec<NodeId>),
    AcknolewdgedAsClient,
}
