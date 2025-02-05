use egui_graphs::Graph;
use petgraph::{csr::DefaultIx, Undirected};

use crate::simulation_controller::{
    edge::{CustomEdgeShape, UiEdgePayload},
    node::{CustomNodeShape, UiNodePayload},
};

pub type UiGraph =
    Graph<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>;
pub type StatusFlag = Option<Result<String, String>>;
