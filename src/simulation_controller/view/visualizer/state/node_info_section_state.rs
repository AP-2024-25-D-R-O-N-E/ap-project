use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};

pub struct NodeInfoSectionState {
    pub opened_windows: HashSet<NodeIndex>,
    // pub node_states: HashMap<NodeIndex, NodeState>,
}

pub struct NodeState {
    last_committed_pdr: f32,
}

impl Default for NodeInfoSectionState {
    fn default() -> Self {
        Self {
            opened_windows: HashSet::new(),
        }
    }
}
