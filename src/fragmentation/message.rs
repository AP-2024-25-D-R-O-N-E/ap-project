use std::mem;

use serde::{Deserialize, Serialize};
use wg_2024::network::NodeId;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Message {
    origin_id: NodeId,
    pub destination_id: NodeId,
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
    // from client to server
    RegisterAsClient(NodeId), // for now lets just use NodeId, maybe in the future we allow for different ids
    UnregisterAsClient(NodeId),
    RequestClients(NodeId),
    RequestHistory {
        requester: NodeId,
        partner: NodeId,
    }, //from is the client requesting, to is the chat partner

    // forwarded from server to client
    TextMessage {
        from: NodeId,
        to: NodeId,
        text: String,
    },
    FileMessage {
        from: NodeId,
        to: NodeId,
        file: Vec<u8>,
        file_name: String,
    }, //filename or file extension?

    // from server to client
    ResponseClients(Vec<NodeId>),
    AcknolewdgedAsClient,
    ResponseHistory {
        partner: NodeId,
        history: Vec<ChatMessage>,
    },
    UnregisteredSenderError,
    UnregisteredRecipientError,

    UnsupportedMessageTypeError, // for example when a client sends response clients to the server
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum ChatMessage {
    TextMessage {
        from: NodeId,
        to: NodeId,
        text: String,
    },
    FileMessage {
        from: NodeId,
        to: NodeId,
        file: Vec<u8>,
        file_name: String,
    }, //file data, file name
}

impl Message {
    pub fn into_u8(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_u8(v: Vec<u8>) -> Self {
        bincode::deserialize(&v).unwrap()
    }
}
