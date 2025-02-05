use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};

use crate::simulation_controller::util::StatusFlag;

pub struct NodeInfoSectionState {
    pub opened_windows: HashMap<NodeIndex, NodeState>,
    // pub node_states: HashMap<NodeIndex, NodeState>,
}

impl NodeInfoSectionState {
    pub fn open_window(&mut self, node: NodeIndex) {
        self.opened_windows.insert(node, Default::default());
    }
}

#[derive(Debug, Clone)]
pub struct NodeState {
    pub last_committed_pdr: f32,
    pub crash_status_flag: StatusFlag,
}
impl Default for NodeState {
    fn default() -> Self {
        Self {
            last_committed_pdr: 0.0,
            crash_status_flag: None,
        }
    }
}

impl Default for NodeInfoSectionState {
    fn default() -> Self {
        Self {
            opened_windows: HashMap::new(),
        }
    }
}
