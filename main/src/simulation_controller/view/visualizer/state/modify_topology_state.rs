use petgraph::{prelude::StableGraph, Undirected};
use wg_2024::network::NodeId;

use crate::{
    initializer::drone_vendor::DroneVendor,
    simulation_controller::{edge::UiEdgePayload, node::UiNodePayload, util::StatusFlag},
};

pub struct ModifyTopologyState {
    pub id: NodeId,
    pub pdr: f32,
    pub neighbors: String,
    pub status_flag: StatusFlag,
    pub drone_vendor: DroneVendor,
}

impl Default for ModifyTopologyState {
    fn default() -> Self {
        Self {
            id: 0,
            pdr: 0.0,
            neighbors: String::new(),
            status_flag: None,
            drone_vendor: DroneVendor::MyDrone,
        }
    }
}

impl ModifyTopologyState {
    pub fn from(graph: &StableGraph<UiNodePayload, UiEdgePayload, Undirected>) -> Self {
        let mut max = 0;
        for x in graph.node_weights() {
            if x.wg_id > max {
                max = x.wg_id;
            }
        }

        Self {
            id: max + 1,
            pdr: 0.0,
            neighbors: String::new(),
            status_flag: None,
            drone_vendor: DroneVendor::MyDrone,
        }
    }
}
