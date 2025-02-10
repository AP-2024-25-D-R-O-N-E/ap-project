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
    fragmentation::message::ChatMessage,
    simulation_controller::{serialization_ref_structs::NodeId, ServerEvent},
};

pub type LockRef<T> = Arc<RwLock<T>>;

pub struct FileDescriptor {
    pub file: Vec<u8>,
    pub file_name: OsString,
    pub extension: OsString,
    pub id: NodeId,
}

impl FileDescriptor {
    pub fn new(file: Vec<u8>, file_name: OsString, extension: OsString, id: NodeId) -> Self {
        Self {
            file,
            file_name,
            extension,
            id,
        }
    }
}

pub struct ServerChannels {
    pub packet_sender: LockRef<HashMap<u8, Sender<Packet>>>,
    pub sim_contr_send: Sender<ServerEvent>,
    pub fragment_recv: Receiver<(NodeId, u64, Fragment)>,
    pub nack_recv: Receiver<Packet>,
}

impl ServerChannels {
    pub fn new(
        packet_sender: LockRef<HashMap<u8, Sender<Packet>>>,
        sim_contr_send: Sender<ServerEvent>,
        fragment_recv: Receiver<(NodeId, u64, Fragment)>,
        nack_recv: Receiver<Packet>,
    ) -> Self {
        Self {
            packet_sender,
            sim_contr_send,
            fragment_recv,
            nack_recv,
        }
    }
}

pub struct ServerTopology {
    pub topology: LockRef<GraphMap<NodeId, (), Directed>>,
    pub edge_nodes: LockRef<HashSet<NodeId>>,
    pub pdr_estimation: LockRef<HashMap<NodeId, (f64, u64, u64)>>,
}

impl ServerTopology {
    pub fn new(
        topology: LockRef<GraphMap<NodeId, (), Directed>>,
        edge_nodes: LockRef<HashSet<NodeId>>,
        pdr_estimation: LockRef<HashMap<NodeId, (f64, u64, u64)>>,
    ) -> Self {
        Self {
            topology,
            edge_nodes,
            pdr_estimation,
        }
    }
}
