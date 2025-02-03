use std::collections::HashMap;

use super::super::node::UiNodeType;
use crossbeam::channel::{unbounded, Receiver, Sender};
use egui_graphs::{events::Event, Graph};
use petgraph::{
    csr::DefaultIx,
    graph::NodeIndex,
    prelude::{StableGraph, StableUnGraph},
    Undirected,
};
use wg_2024::network::NodeId;

use crate::simulation_controller::{
    edge::{CustomEdgeShape, UiEdgePayload},
    node::{CustomNodeShape, UiClientNode, UiDroneNode, UiNodePayload, UiServerNode},
    util::is_well_formed,
};

pub struct GraphSectionState {
    pub g: Graph<
        UiNodePayload,
        UiEdgePayload,
        Undirected,
        DefaultIx,
        CustomNodeShape,
        CustomEdgeShape,
    >,

    pub graph_event_publisher: Sender<Event>,
    pub graph_event_consumer: Receiver<Event>,
    pub well_formedness_flag: Result<(), String>,

    pub node_id_map: HashMap<wg_2024::network::NodeId, petgraph::graph::NodeIndex>,
}

fn get_node_id_map(
    graph: &StableGraph<UiNodePayload, UiEdgePayload, Undirected>,
) -> HashMap<u8, NodeIndex> {
    let mut map = HashMap::new();

    for n in graph.node_indices() {
        let payload = graph.node_weight(n).unwrap();
        map.insert(payload.wg_id, n);
    }
    map
}
impl GraphSectionState {
    fn new(graph: StableGraph<UiNodePayload, UiEdgePayload, Undirected>) -> GraphSectionState {
        let graph_section = GraphSectionState::default();
        let (event_publisher, event_consumer) = unbounded();
        let ui_graph = Graph::from(&graph);
        let well_formedness_flag = is_well_formed(&ui_graph.g);

        GraphSectionState {
            g: ui_graph,
            graph_event_publisher: event_publisher,
            graph_event_consumer: event_consumer,
            node_id_map: get_node_id_map(&graph),
            well_formedness_flag,
        }
    }

    pub fn wg_id(&self, index: NodeIndex) -> Option<NodeId> {
        match self.g.node(index) {
            Some(node) => Some(node.payload().wg_id),
            None => None,
        }
    }

    pub fn graph_id(&self, index: NodeId) -> Option<NodeIndex> {
        match self.node_id_map.get(&index) {
            Some(idx) => Some(*idx),
            None => None,
        }
    }
}

impl Default for GraphSectionState {
    fn default() -> Self {
        let (event_publisher, event_consumer) = unbounded();
        let graph = generate_graph();
        Self {
            g: Graph::from(&graph),
            graph_event_publisher: event_publisher,
            graph_event_consumer: event_consumer,
            node_id_map: get_node_id_map(&graph),
            well_formedness_flag: Ok(()),
        }
    }
}

fn generate_graph() -> StableGraph<UiNodePayload, UiEdgePayload, Undirected> {
    let mut graph = StableUnGraph::<UiNodePayload, UiEdgePayload>::default();

    let a = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Client(UiClientNode {}),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 0,
    });
    let b = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 1,
    });
    let c = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 2,
    });
    let d = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Server(UiServerNode {}),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 3,
    });
    let e = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 4,
    });
    let f = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 5,
    });
    let g = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::default()),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 6,
    });
    let h = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Server(UiServerNode {}),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 7,
    });
    let i = graph.add_node(UiNodePayload {
        node_type: UiNodeType::Server(UiServerNode {}),
        vendor: "d_r_o_n_e".to_string(),
        wg_id: 8,
    });

    graph.add_edge(a, b, UiEdgePayload::default());

    graph.add_edge(b, c, UiEdgePayload::default());
    graph.add_edge(b, e, UiEdgePayload::default());

    graph.add_edge(e, c, UiEdgePayload::default());
    graph.add_edge(e, f, UiEdgePayload::default());
    graph.add_edge(e, g, UiEdgePayload::default());
    graph.add_edge(e, i, UiEdgePayload::default());

    graph.add_edge(g, h, UiEdgePayload::default());
    graph.add_edge(g, f, UiEdgePayload::default());

    graph.add_edge(i, h, UiEdgePayload::default());

    graph.add_edge(f, c, UiEdgePayload::default());
    graph.add_edge(f, d, UiEdgePayload::default());

    graph.add_edge(c, d, UiEdgePayload::default());

    graph
}
