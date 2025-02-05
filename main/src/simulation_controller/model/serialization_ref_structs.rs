use macros::IntoSerializable;
// use crossbeam::channel::Sender;
// use wg_2024::network::NodeId as WGNodeId;
// use wg_2024::network::SourceRoutingHeader as wgSourceRoutingHeader;
// use wg_2024::packet::NodeType as WGNodeType;
// use wg_2024::packet::{Packet as WGPacket, PacketType as WGPacketType};
//
//
use wg_2024::network::*;
use wg_2024::packet::*;

// use serde::Serialize;
// use wg_2024::network::NodeId;
// use wg_2024::packet::FloodResponse;
// use wg_2024::packet::NodeType as WGNodeType;

// use crate::{FloodRequest, FloodResponse};
// use std::fmt::{Debug, Display, Formatter};
// use wg_network::{NodeId, SourceRoutingHeader};

use serde::Serialize;
pub const FRAGMENT_DSIZE: usize = 128;

pub type NodeId = u8;

impl IntoSerializable<usize> for usize {
    fn into_serializable(&self) -> Self {
        self.clone()
    }
}
impl IntoSerializable<u8> for u8 {
    fn into_serializable(&self) -> Self {
        self.clone()
    }
}

impl IntoSerializable<Vec<NodeId>> for Vec<NodeId> {
    fn into_serializable(&self) -> Self {
        self.clone()
    }
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct SourceRoutingHeaderRef {
    pub hop_index: usize,
    pub hops: Vec<NodeId>,
}

#[derive(Debug, Clone, IntoSerializable)]
pub struct PacketRef {
    pub routing_header: SourceRoutingHeaderRef,
    pub session_id: u64,
    pub pack_type: PacketTypeRef,
}

#[derive(IntoSerializable, Debug, Clone)]
pub enum PacketTypeRef {
    MsgFragment(FragmentRef),
    Ack(AckRef),
    Nack(NackRef),
    FloodRequest(FloodRequestRef),
    FloodResponse(FloodResponseRef),
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct NackRef {
    pub fragment_index: u64, // If the packet is not a fragment, it's considered as a whole, so fragment_index will be 0.
    pub nack_type: NackTypeRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// #[derive(IntoSerializable, Debug, Clone)]
pub enum NackTypeRef {
    ErrorInRouting(NodeId), // contains id of not neighbor
    DestinationIsDrone,
    Dropped,
    UnexpectedRecipient(NodeId),
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct AckRef {
    pub fragment_index: u64,
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct FragmentRef {
    pub fragment_index: u64,
    pub total_n_fragments: u64,
    pub length: u8,
    pub data: [u8; FRAGMENT_DSIZE],
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct FloodRequestRef {
    pub flood_id: u64,
    pub initiator_id: NodeId,
    pub path_trace: Vec<(NodeId, NodeTypeRef)>,
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct FloodResponseRef {
    pub flood_id: u64,

    // #[serde(skip)]
    pub path_trace: Vec<(NodeId, NodeTypeRef)>,
}

#[derive(IntoSerializable, Debug, Clone)]
pub enum NodeTypeRef {
    Client,
    Drone,
    Server,
}

pub trait IntoSerializable<T> {
    fn into_serializable(&self) -> T;
}
//
// impl IntoSerializable for WGPacket {
//     fn into_serializable(&self) -> Self {
//         todo!()
//     }
// }
