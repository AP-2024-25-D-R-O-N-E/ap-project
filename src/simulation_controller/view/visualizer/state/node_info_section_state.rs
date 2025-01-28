use petgraph::graph::NodeIndex;
use std::collections::HashSet;

pub struct NodeInfoSectionState {
    pub opened_windows: HashSet<NodeIndex>,
}

impl Default for NodeInfoSectionState {
    fn default() -> Self {
        Self {
            opened_windows: HashSet::new(),
        }
    }
}
