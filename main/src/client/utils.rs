use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    sync::{Arc, Mutex, RwLock},
};

use crossbeam::channel::{Receiver, Sender};
use petgraph::{prelude::GraphMap, Directed};
use tempfile::TempDir;
use wg_2024::packet::{Fragment, Packet};

use crate::{
    fragmentation::message::{ChatMessage, Message},
    simulation_controller::{serialization_ref_structs::NodeId, ClientEvent, ServerEvent},
};

pub type LockRef<T> = Arc<RwLock<T>>;

pub struct ClientChannels {
    pub command_recv: Receiver<Message>,
    pub fragment_sender: Sender<(NodeId, u64, Fragment)>,
    pub sim_send: Sender<ClientEvent>,
}

impl ClientChannels {
    pub fn new(
        command_recv: Receiver<Message>,
        fragment_sender: Sender<(NodeId, u64, Fragment)>,
        sim_send: Sender<ClientEvent>,
    ) -> Self {
        Self {
            command_recv,
            fragment_sender,
            sim_send,
        }
    }
}

pub struct SenderThreadChannels {
    pub packet_sender: LockRef<HashMap<u8, Sender<Packet>>>,
    pub sim_contr_send: Sender<ClientEvent>,
    pub nack_recv: Receiver<Packet>,
    pub fragment_recv: Receiver<(NodeId, u64, Fragment)>,
}

impl SenderThreadChannels {
    pub fn new(
        packet_sender: LockRef<HashMap<u8, Sender<Packet>>>,
        sim_contr_send: Sender<ClientEvent>,
        nack_recv: Receiver<Packet>,
        fragment_recv: Receiver<(NodeId, u64, Fragment)>,
    ) -> Self {
        Self {
            packet_sender,
            sim_contr_send,
            nack_recv,
            fragment_recv,
        }
    }
}
