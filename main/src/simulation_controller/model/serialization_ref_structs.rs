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
// pub const FRAGMENT_DSIZE: usize = 128;
//
pub type NodeId = u8;

pub trait IntoSerializable {
    type Output;
    fn into_serializable(&self) -> Self::Output;
}

impl IntoSerializable for usize {
    type Output = usize;
    fn into_serializable(&self) -> Self::Output {
        *self
    }
}

// impl IntoSerializable for u8 {
//     type Output = u8;
//     fn into_serializable(&self) -> Self::Output {
//         *self
//     }
// }

impl IntoSerializable for NodeId {
    type Output = NodeId;
    fn into_serializable(&self) -> Self::Output {
        self.clone()
    }
}

impl IntoSerializable for u64 {
    type Output = u64;
    fn into_serializable(&self) -> Self::Output {
        *self
    }
}

impl<const N: usize> IntoSerializable for [u8; N] {
    type Output = [u8; N];
    fn into_serializable(&self) -> Self::Output {
        *self
    }
}

impl<T, U> IntoSerializable for (T, U)
where
    T: Clone + IntoSerializable,
    U: Clone + IntoSerializable,
{
    type Output = (
        <T as IntoSerializable>::Output,
        <U as IntoSerializable>::Output,
    );
    fn into_serializable(&self) -> Self::Output {
        (self.0.into_serializable(), self.1.into_serializable())
    }
}

impl<T> IntoSerializable for Vec<T>
where
    T: Clone + IntoSerializable,
{
    type Output = Vec<<T as IntoSerializable>::Output>;
    fn into_serializable(&self) -> Self::Output {
        self.iter().map(|item| item.into_serializable()).collect()
    }
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct SourceRoutingHeaderRef {
    pub hop_index: usize,
    pub hops: Vec<NodeId>,
}

#[derive(IntoSerializable, Debug, Clone)]
pub struct PacketRef {
    pub routing_header: SourceRoutingHeaderRef,
    pub session_id: u64,
    pub pack_type: PacketTypeRef,
}

#[derive(Debug, Clone, IntoSerializable)]
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

#[derive(IntoSerializable, Debug, Clone, Copy, PartialEq, Eq)]
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

//
// impl IntoSerializable for WGPacket {
//     fn into_serializable(&self) -> Self {
//         todo!()
//     }
// }
