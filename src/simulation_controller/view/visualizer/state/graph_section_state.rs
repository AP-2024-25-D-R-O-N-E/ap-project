use super::super::node::NodeType;
use crossbeam::channel::{unbounded, Receiver, Sender};
use egui_graphs::{events::Event, Graph};
use petgraph::{
    csr::DefaultIx,
    prelude::{StableGraph, StableUnGraph},
    Undirected,
};

use crate::simulation_controller::node::{
    ClientNode, CustomNodeShape, DroneNode, NodePayload, ServerNode,
};

pub struct GraphSectionState {
    pub g: Graph<NodePayload, (), Undirected, DefaultIx, CustomNodeShape>,

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

fn generate_graph() -> StableGraph<NodePayload, (), Undirected> {
    let mut graph = StableUnGraph::<NodePayload, ()>::default();

    let a = graph.add_node(NodePayload {
        node_type: NodeType::Client(ClientNode {}),
    });
    let b = graph.add_node(NodePayload {
        node_type: NodeType::Drone(DroneNode {}),
    });
    let c = graph.add_node(NodePayload {
        node_type: NodeType::Drone(DroneNode {}),
    });
    let d = graph.add_node(NodePayload {
        node_type: NodeType::Server(ServerNode {}),
    });
    let e = graph.add_node(NodePayload {
        node_type: NodeType::Drone(DroneNode {}),
    });
    let f = graph.add_node(NodePayload {
        node_type: NodeType::Drone(DroneNode {}),
    });
    let g = graph.add_node(NodePayload {
        node_type: NodeType::Drone(DroneNode {}),
    });
    let h = graph.add_node(NodePayload {
        node_type: NodeType::Server(ServerNode {}),
    });
    let i = graph.add_node(NodePayload {
        node_type: NodeType::Server(ServerNode {}),
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
