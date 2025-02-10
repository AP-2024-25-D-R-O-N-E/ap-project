use std::path::PathBuf;

use crossbeam::channel::Sender;
use macros::IntoSerializable;

use crate::fragmentation::message::ChatMessage;

use super::structs::*;
use wg_2024::controller::*;
use wg_2024::network::*;
use wg_2024::packet::*;

use serde::ser::{SerializeStruct, Serializer};
use serde::Serialize;

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

impl IntoSerializable for NodeId {
    type Output = NodeId;
    fn into_serializable(&self) -> Self::Output {
        *self
    }
}

impl IntoSerializable for u64 {
    type Output = u64;
    fn into_serializable(&self) -> Self::Output {
        *self
    }
}

impl IntoSerializable for String {
    type Output = String;
    fn into_serializable(&self) -> Self::Output {
        self.clone()
    }
}

impl<const N: usize> IntoSerializable for [u8; N] {
    type Output = [u8; N];
    fn into_serializable(&self) -> Self::Output {
        *self
    }
}

impl IntoSerializable for PathBuf {
    type Output = PathBuf;
    fn into_serializable(&self) -> Self::Output {
        self.clone()
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

impl<T> IntoSerializable for Sender<T>
where
    T: Clone,
{
    type Output = Sender<T>;
    fn into_serializable(&self) -> Self::Output {
        self.clone()
    }
}

#[derive(Serialize, IntoSerializable, Debug, Clone)]
pub struct SourceRoutingHeaderRef {
    pub hop_index: usize,
    pub hops: Vec<NodeId>,
}

#[derive(Serialize, IntoSerializable, Debug, Clone)]
pub struct PacketRef {
    pub routing_header: SourceRoutingHeaderRef,
    pub session_id: u64,
    pub pack_type: PacketTypeRef,
}

#[derive(IntoSerializable, Serialize, Debug, Clone)]
pub enum PacketTypeRef {
    MsgFragment(FragmentRef),
    Ack(AckRef),
    Nack(NackRef),
    FloodRequest(FloodRequestRef),
    FloodResponse(FloodResponseRef),
}

#[derive(Serialize, IntoSerializable, Debug, Clone)]
pub struct NackRef {
    pub fragment_index: u64, // If the packet is not a fragment, it's considered as a whole, so fragment_index will be 0.
    pub nack_type: NackTypeRef,
}

#[derive(Serialize, IntoSerializable, Debug, Clone, Copy, PartialEq, Eq)]
pub enum NackTypeRef {
    ErrorInRouting(NodeId), // contains id of not neighbor
    DestinationIsDrone,
    Dropped,
    UnexpectedRecipient(NodeId),
}

#[derive(Serialize, IntoSerializable, Debug, Clone)]
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

#[derive(Serialize, IntoSerializable, Debug, Clone)]
pub struct FloodRequestRef {
    pub flood_id: u64,
    pub initiator_id: NodeId,
    pub path_trace: Vec<(NodeId, NodeTypeRef)>,
}

#[derive(Serialize, IntoSerializable, Debug, Clone)]
pub struct FloodResponseRef {
    pub flood_id: u64,

    // #[serde(skip)]
    pub path_trace: Vec<(NodeId, NodeTypeRef)>,
}

#[derive(Serialize, IntoSerializable, Debug, Clone)]
pub enum NodeTypeRef {
    Client,
    Drone,
    Server,
}

impl Serialize for FragmentRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut first_5 = String::new();
        let mut last_5 = String::new();
        let v = &self.data;
        for item in v.iter().take(4) {
            first_5.push_str(format!("{},", item).as_str());
        }
        first_5.push_str(format!("{}", v[5]).as_str());

        for item in v.iter().take(v.len() - 1).skip(v.len() - 5) {
            last_5.push_str(format!("{},", item).as_str());
        }
        last_5.push_str(format!("{}", v[v.len() - 1]).as_str());

        let compact_data_rep = format!("[{}, . . , {}]", first_5, last_5);

        let mut state = serializer.serialize_struct("FragmentRef", 4)?;
        state.serialize_field("fragment_index", &self.fragment_index)?;
        state.serialize_field("total_n_fragments", &self.total_n_fragments)?;
        state.serialize_field("length", &self.length)?;
        state.serialize_field("data", &compact_data_rep)?; // Convert array to slice
        state.end()
    }
}

#[derive(IntoSerializable, Serialize, Debug, Clone)]
pub enum ClientEventRef {
    PacketSent(PacketRef),
    PacketReceived(PacketRef),
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

    CreatedFileLocal{
        from: NodeId,
        to: NodeId,
        file_path: PathBuf,
    },

    ResponseClientsReceived(Vec<NodeId>),
    AcknolewdgedAsClient,
    ResponseHistoryReceived {
        partner: NodeId,
        history: Vec<ChatMessageRef>,
    },
    UnregisteredSenderError,
    UnregisteredRecipientError,
    UnsupportedMessageTypeError,
}

#[derive(IntoSerializable, Serialize, Debug, PartialEq, Clone)]
pub enum ChatMessageRef {
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

#[derive(IntoSerializable, Serialize, Debug, Clone)]
pub enum ServerEventRef {
    PacketSent(PacketRef),
    PacketReceived(PacketRef),
}

#[derive(IntoSerializable, Serialize, Debug, Clone)]
pub enum DroneEventRef {
    PacketSent(PacketRef),
    PacketDropped(PacketRef),
    ControllerShortcut(PacketRef),
}

#[derive(IntoSerializable, Serialize, Debug, Clone)]
pub enum SCEventTypeRef {
    Client(ClientEventRef),
    Server(ServerEventRef),
    Drone(DroneEventRef),
}

#[derive(IntoSerializable, Serialize, Debug, Clone)]
pub struct SCEventRef {
    pub event_type: SCEventTypeRef,
    pub sender_id: NodeId,
}
