use egui_graphs::{Edge, Graph, Node};
use petgraph::{csr::DefaultIx, prelude::StableGraph, Undirected};

use crate::simulation_controller::{
    edge::{CustomEdgeShape, UiEdgePayload},
    node::{CustomNodeShape, UiNodePayload},
};

pub type UiGraph =
    Graph<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>;

pub type ModelGraph<N, E> = StableGraph<N, E, Undirected>;

pub type DefaultModelGraph = StableGraph<ModelNodePayload, ModelEdgePayload, Undirected>;

pub type ModelNodePayload =
    Node<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape>;

pub type ModelEdgePayload =
    Edge<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>;

pub type StatusFlag = Option<Result<String, String>>;
