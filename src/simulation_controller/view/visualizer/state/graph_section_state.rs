use super::super::node::UiNodeType;
use crossbeam::channel::{unbounded, Receiver, Sender};
use egui_graphs::{events::Event, Graph};
use petgraph::{
    csr::DefaultIx,
    prelude::{StableGraph, StableUnGraph},
    Undirected,
};

use crate::simulation_controller::node::{
    CustomNodeShape, UiClientNode, UiDroneNode, UiNodePayload, UiServerNode,
};

pub struct GraphSectionState {
    pub g: Graph<UiNodePayload, (), Undirected, DefaultIx, CustomNodeShape>,

    pub graph_event_publisher: Sender<Event>,
    pub graph_event_consumer: Receiver<Event>,
}

impl Default for GraphSectionState {
    fn default() -> Self {
        let (event_publisher, event_consumer) = unbounded();
        Self {
            g: Graph::from(&generate_graph()),
            graph_event_publisher: event_publisher,
            graph_event_consumer: event_consumer,
        }
    }
}

fn generate_graph() -> StableGraph<UiNodePayload, (), Undirected> {
    let mut graph = StableUnGraph::<UiNodePayload, ()>::default();

    let a = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Client(UiClientNode {}),
        vendor: "d_r_o_n_e".to_string(),
    });
    let b = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
    });
    let c = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
    });
    let d = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Server(UiServerNode {}),
        vendor: "d_r_o_n_e".to_string(),
    });
    let e = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
    });
    let f = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
    });
    let g = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
    });
    let h = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Server(UiServerNode {}),
        vendor: "d_r_o_n_e".to_string(),
    });
    let i = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Server(UiServerNode {}),
        vendor: "d_r_o_n_e".to_string(),
    });

    graph.add_edge(a, b, ());

    graph.add_edge(b, c, ());
    graph.add_edge(b, e, ());

    graph.add_edge(e, c, ());
    graph.add_edge(e, f, ());
    graph.add_edge(e, g, ());
    graph.add_edge(e, i, ());

    graph.add_edge(g, h, ());
    graph.add_edge(g, f, ());

    graph.add_edge(i, h, ());

    graph.add_edge(f, c, ());
    graph.add_edge(f, d, ());

    graph.add_edge(c, d, ());

    graph
}
