use std::{ffi::OsString, path::PathBuf};

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
        file_name: OsString,
        extension: OsString,
    }, //filename or file extension?

    // from server to client
    ResponseClients(Vec<NodeId>),
    AcknolewdgedAsClient,
    ResponseHistory {
        partner: NodeId,
        history: Vec<RawChatMessage>,
    },
    UnregisteredSenderError,
    UnregisteredRecipientError,

    UnsupportedMessageTypeError, // for example when a client sends response clients to the server
}

// raw one is sent by the server to the client
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum RawChatMessage {
    TextMessage {
        from: NodeId,
        to: NodeId,
        text: String,
    },
    FileMessage {
        from: NodeId,
        to: NodeId,
        file: Vec<u8>,
        file_name: OsString,
        extension: OsString,
    }, //file data, file name
}

// this one is sent by the client to the simulation controller
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum ChatMessage {
    TextMessage {
        from: NodeId,
        to: NodeId,
        text: String,
    },
    FileMessage {
        from: NodeId,
        to: NodeId,
        file_path: PathBuf,
    },
}

impl Message {
    pub fn as_u8(&self) -> Vec<u8> {
        bincode::serialize(&self).unwrap()
    }

    pub fn from_u8(v: Vec<u8>) -> Self {
        bincode::deserialize(&v).unwrap_or(Message {
            origin_id: 0,
            destination_id: 0,
            message_data: MessageData::UnsupportedMessageTypeError,
        })
    }
}
