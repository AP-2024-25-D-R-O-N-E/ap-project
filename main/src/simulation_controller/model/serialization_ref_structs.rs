// use crossbeam::channel::Sender;
// use wg_2024::network::NodeId as WGNodeId;
// use wg_2024::network::SourceRoutingHeader as wgSourceRoutingHeader;
// use wg_2024::packet::NodeType as WGNodeType;
// use wg_2024::packet::{Packet as WGPacket, PacketType as WGPacketType};
//
//

// use serde::Serialize;
// use wg_2024::network::NodeId;
// use wg_2024::packet::FloodResponse;
use wg_2024::packet::NodeType as WGNodeType;

// use crate::{FloodRequest, FloodResponse};
// use std::fmt::{Debug, Display, Formatter};
// use wg_network::{NodeId, SourceRoutingHeader};

use serde::Serialize;
pub const FRAGMENT_DSIZE: usize = 128;

pub type NodeId = u8;

#[derive(Debug, Clone)]
pub struct SourceRoutingHeader {
    pub hop_index: usize,
    pub hops: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct Packet {
    pub routing_header: SourceRoutingHeader,
    pub session_id: u64,
    pub pack_type: PacketType,
}

#[derive(Debug, Clone)]
pub enum PacketType {
    MsgFragment(Fragment),
    Ack(Ack),
    Nack(Nack),
    FloodRequest(FloodRequest),
    FloodResponse(FloodResponse),
}

#[derive(Debug, Clone)]
pub struct Nack {
    pub fragment_index: u64, // If the packet is not a fragment, it's considered as a whole, so fragment_index will be 0.
    pub nack_type: NackType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NackType {
    ErrorInRouting(NodeId), // contains id of not neighbor
    DestinationIsDrone,
    Dropped,
    UnexpectedRecipient(NodeId),
}

#[derive(Debug, Clone)]
pub struct Ack {
    pub fragment_index: u64,
}

#[derive(Debug, Clone)]
pub struct Fragment {
    pub fragment_index: u64,
    pub total_n_fragments: u64,
    pub length: u8,
    pub data: [u8; FRAGMENT_DSIZE],
}

#[derive(Debug, Clone)]
pub struct FloodRequest {
    pub flood_id: u64,
    pub initiator_id: NodeId,
    pub path_trace: Vec<(NodeId, NodeType)>,
}

#[derive(Debug, Clone)]
pub struct FloodResponse {
    pub flood_id: u64,

    // #[serde(skip)]
    pub path_trace: Vec<(NodeId, NodeType)>,
}

#[derive(Serialize, Debug, Clone)]
pub enum NodeType {
    Client,
    Drone,
    Server,
}

impl From<WGNodeType> for NodeType {
    fn from(node_type: WGNodeType) -> Self {
        match node_type {
            WGNodeType::Client => Self::Client,
            WGNodeType::Drone => Self::Drone,
            WGNodeType::Server => Self::Server,
        }
    }
}

pub trait IntoSerializable<T> {
    fn into_serializable(&self) -> Self;
}
//
// impl IntoSerializable for WGPacket {
//     fn into_serializable(&self) -> Self {
//         todo!()
//     }
// }
