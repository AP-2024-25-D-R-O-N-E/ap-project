use petgraph::{graph::NodeIndex, prelude::StableGraph, Undirected};
use std::collections::{HashMap, HashSet};
use wg_2024::network::NodeId;

use crate::simulation_controller::{
    edge::UiEdgePayload,
    node::{
        UiNodePayload,
        UiNodeType::{Client, Drone, Server},
    },
    util::StatusFlag,
};

#[derive(Default)]
pub struct NodeInfoSectionState {
    pub opened_windows: HashSet<NodeIndex>,
    pub states: HashMap<NodeIndex, NodeState>,
    // pub opened_clients: HashMap<NodeIndex, NodeState>,
    // pub opened_servers: HashMap<NodeIndex, NodeState>,
    // pub node_states: HashMap<NodeIndex, NodeState>,
}

impl NodeInfoSectionState {
    pub fn from(graph: &StableGraph<UiNodePayload, UiEdgePayload, Undirected>) -> Self {
        let mut states = HashMap::new();
        for node_index in graph.node_indices() {
            let node_payload = graph.node_weight(node_index).unwrap();
            match &node_payload.node_type {
                Server(ui_server_node) => {
                    states.insert(node_index, NodeState::Server(ServerState {}));
                }
                Client(ui_client_node) => {
                    states.insert(node_index, NodeState::Client(ClientState::default()));
                }
                Drone(ui_drone_node) => {
                    states.insert(
                        node_index,
                        NodeState::Drone(DroneState {
                            last_committed_pdr: ui_drone_node.pdr,
                            crash_status_flag: StatusFlag::default(),
                        }),
                    );
                }
            }
        }

        Self {
            opened_windows: HashSet::new(),
            states,
        }
    }
    pub fn open_window(&mut self, node: NodeIndex) {
        self.opened_windows.insert(node);
    }
}

#[derive(Debug, Clone)]
pub struct DroneState {
    pub last_committed_pdr: f32,
    pub crash_status_flag: StatusFlag,
}

#[derive(Debug, Clone)]
#[derive(Default)]
pub struct ClientState {
    pub available_peers: Vec<NodeId>,
    pub curr_msg: String,
    pub current_peer: Option<NodeId>,
}

#[derive(Debug, Clone)]
pub struct ServerState {}

#[derive(Debug, Clone)]
pub enum NodeState {
    Client(ClientState),
    Server(ServerState),
    Drone(DroneState),
}


impl Default for NodeState {
    fn default() -> Self {
        Self::Drone(DroneState {
            last_committed_pdr: 0.0,
            crash_status_flag: StatusFlag::default(),
        })
    }
}


impl<'a> TryInto<&'a mut DroneState> for &'a mut NodeState {
    type Error = &'static str;
    fn try_into(self) -> Result<&'a mut DroneState, Self::Error> {
        match self {
            NodeState::Drone(drone_state) => Ok(drone_state),
            _ => Err("NodeState is not a Drone"),
        }
    }
}

impl<'a> TryInto<&'a mut ClientState> for &'a mut NodeState {
    type Error = &'static str;
    fn try_into(self) -> Result<&'a mut ClientState, Self::Error> {
        match self {
            NodeState::Client(client_state) => Ok(client_state),
            _ => Err("NodeState is not a Client"),
        }
    }
}
impl<'a> TryInto<&'a mut ServerState> for &'a mut NodeState {
    type Error = &'static str;
    fn try_into(self) -> Result<&'a mut ServerState, Self::Error> {
        match self {
            NodeState::Server(server_state) => Ok(server_state),
            _ => Err("NodeState is not a Server"),
        }
    }
}
