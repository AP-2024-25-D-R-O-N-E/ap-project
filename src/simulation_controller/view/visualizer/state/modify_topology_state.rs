use wg_2024::network::NodeId;

use crate::simulation_controller::util::StatusFlag;

pub struct ModifyTopologyState {
    pub id: NodeId,
    pub pdr: f32,
    pub neighbors: String,
    pub status_flag: StatusFlag,
}

impl Default for ModifyTopologyState {
    fn default() -> Self {
        Self {
            id: 0,
            pdr: 0.0,
            neighbors: String::new(),
            status_flag: None,
        }
    }
}
