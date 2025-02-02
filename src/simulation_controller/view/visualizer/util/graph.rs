use egui_graphs::{Edge, Graph, Node};
use petgraph::{
    csr::DefaultIx,
    graph::{EdgeIndex, NodeIndex},
    prelude::StableGraph,
    Undirected,
};
use std::collections::{HashSet, VecDeque};

use crate::simulation_controller::{
    edge::{CustomEdgeShape, UiEdgePayload},
    node::{CustomNodeShape, UiNodePayload, UiNodeType},
    state::State,
    util::*,
    SimulationController,
};

pub fn check_edge_removal(
    status_flag: &mut Option<Result<String, String>>,
    graph: &mut Graph<
        UiNodePayload,
        UiEdgePayload,
        Undirected,
        DefaultIx,
        CustomNodeShape,
        CustomEdgeShape,
    >,
    node1: NodeIndex,
    node2: NodeIndex,
) -> bool {
    let mut edges = HashSet::new();
    for (edge_index, edge) in graph.edges_connecting(node1, node2) {
        edges.insert(edge_index);
    }

    let node1_payload = graph.node(node1).unwrap().payload().clone();
    let node2_payload = graph.node(node2).unwrap().payload().clone();

    // Check if an edge exists between the two nodes
    if edges.is_empty() {
        *status_flag = Some(Err(format!(
            "No channel between {} and {}",
            node1_payload.wg_id, node1_payload.wg_id
        )));
        return false;
    }
    // Check if removing this edge would disconnect the graph
    else if !is_connected_without_edge_set(&graph.g, edges) {
        *status_flag = Some(Err(
            "Removing this edge would cause the graph to be disconnected".to_string(),
        ));
        return false;
    }
    // Check if the server has at least 2 connections
    else if !match (node1_payload.node_type, node2_payload.node_type) {
        (
            crate::simulation_controller::node::UiNodeType::Drone(_),
            crate::simulation_controller::node::UiNodeType::Server(_),
        ) => {
            let neighbors: Vec<NodeIndex> = graph.g.neighbors_undirected(node2).collect();
            if neighbors.len() == 2 {
                false
            } else {
                true
            }
        }
        (
            crate::simulation_controller::node::UiNodeType::Server(_),
            crate::simulation_controller::node::UiNodeType::Drone(_),
        ) => {
            let neighbors: Vec<NodeIndex> = graph.g.neighbors_undirected(node1).collect();
            if neighbors.len() == 2 {
                false
            } else {
                true
            }
        }
        _ => true,
    } {
        *status_flag = Some(Err(
            "Cannot remove edge: each server should be connected to at least 2 nodes".to_string(),
        ));
        return false;
    }

    true
}

pub fn remove_edges_between(
    graph: &mut UiGraph,
    status_flag: &mut StatusFlag,
    node1: NodeIndex,
    node2: NodeIndex,
    simulation_controller: &SimulationController,
) {
    graph.remove_edges_between(node1, node2);
    *status_flag = Some(Ok("Sender removed with success".to_string()));

    let node1_wg_id = graph.node(node1).unwrap().payload().wg_id;
    let node2_wg_id = graph.node(node2).unwrap().payload().wg_id;

    simulation_controller.send_remove_sender_command(node1_wg_id, node2_wg_id);
    simulation_controller.send_remove_sender_command(node2_wg_id, node1_wg_id);
}

pub fn check_edge_addition(
    graph: &mut UiGraph,
    status_flag: &mut StatusFlag,
    node1: NodeIndex,
    node2: NodeIndex,
) -> bool {
    // Ensure channel does not already exist
    let edges: HashSet<_> = graph
        .edges_connecting(node1, node2)
        .map(|(edge_index, _)| edge_index)
        .collect();
    if edges.len() > 0 {
        *status_flag = Some(Err(
            "Channel already exists between the two nodes".to_string()
        ));
        return false;
    }

    // Helper function to validate client connection
    fn check_drone_client(
        graph: &mut UiGraph,
        status_flag: &mut StatusFlag,
        client: NodeIndex,
    ) -> bool {
        let neighbors: Vec<NodeIndex> = graph.g.neighbors_undirected(client).collect();
        if neighbors.len() >= 2 {
            *status_flag = Some(Err("Client nodes can have at most 2 neighbors".to_string()));
            return false;
        }
        true
    }

    // Helper function to handle non-drone connections
    fn check_non_drone(status_flag: &mut Option<Result<String, String>>) -> bool {
        *status_flag = Some(Err("Cannot have link between two non-drones".to_string()));
        false
    }

    // Check client contraints
    let node1_payload = match graph.node(node1) {
        Some(node) => node.payload().clone(),
        None => return false, // Early return if node1 is invalid
    };

    let node2_payload = match graph.node(node2) {
        Some(node) => node.payload().clone(),
        None => return false, // Early return if node2 is invalid
    };
    match (node1_payload.node_type, node2_payload.node_type) {
        (
            crate::simulation_controller::node::UiNodeType::Drone(_),
            crate::simulation_controller::node::UiNodeType::Client(_),
        ) => check_drone_client(graph, status_flag, node2),
        (
            crate::simulation_controller::node::UiNodeType::Client(_),
            crate::simulation_controller::node::UiNodeType::Drone(_),
        ) => check_drone_client(graph, status_flag, node1),
        (
            crate::simulation_controller::node::UiNodeType::Server(_),
            crate::simulation_controller::node::UiNodeType::Client(_),
        )
        | (
            crate::simulation_controller::node::UiNodeType::Client(_),
            crate::simulation_controller::node::UiNodeType::Server(_),
        )
        | (
            crate::simulation_controller::node::UiNodeType::Client(_),
            crate::simulation_controller::node::UiNodeType::Client(_),
        )
        | (
            crate::simulation_controller::node::UiNodeType::Server(_),
            crate::simulation_controller::node::UiNodeType::Server(_),
        ) => check_non_drone(status_flag),
        _ => true,
    }
}

pub fn add_edge_between(
    graph: &mut UiGraph,
    status_flag: &mut StatusFlag,
    node1: NodeIndex,
    node2: NodeIndex,
    simulation_controller: &SimulationController,
) {
    graph.add_edge(node1, node2, UiEdgePayload::default());
    *status_flag = Some(Ok("Sender added with success".to_string()));
    let node1_wg_id = graph.node(node1).unwrap().payload().wg_id;
    let node2_wg_id = graph.node(node1).unwrap().payload().wg_id;

    simulation_controller.send_add_sender_command(node1_wg_id, node2_wg_id);
    simulation_controller.send_add_sender_command(node2_wg_id, node1_wg_id);
}

pub fn bfs_with_disabled_edges<N>(
    graph: &StableGraph<
        N,
        // Node<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape>,
        Edge<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>,
        Undirected,
    >,
    start: NodeIndex,
) -> usize
where
{
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    queue.push_back(start);

    while (!queue.is_empty()) {
        let curr_node = queue.pop_front().unwrap();

        if (visited.contains(&curr_node)) {
            continue;
        }

        for neighbor in graph.neighbors_undirected(curr_node) {
            let mut has_active_edge = false;
            for edge in graph.edges_connecting(curr_node, neighbor) {
                if (edge.weight().payload().is_active) {
                    has_active_edge = true;
                    break;
                }
            }

            if !visited.contains(&neighbor) && has_active_edge {
                queue.push_back(neighbor);
            }
        }

        visited.insert(curr_node);
    }

    // println!("Visited count: {}", visited_count);
    visited.len()
}

pub fn is_connected_without_edge_set<N>(
    graph: &StableGraph<
        N,
        Edge<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>,
        Undirected,
    >,
    excluded_edges: HashSet<EdgeIndex>,
) -> bool
where
{
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let first_node;
    match graph.node_indices().next() {
        Some(first_node_unwrapped) => first_node = first_node_unwrapped,
        None => return true,
    }
    queue.push_back(first_node);

    while (!queue.is_empty()) {
        let curr_node = queue.pop_front().unwrap();

        if (visited.contains(&curr_node)) {
            continue;
        }

        for neighbor in graph.neighbors_undirected(curr_node) {
            let mut has_active_edge = false;
            for edge in graph.edges_connecting(curr_node, neighbor) {
                if edge.weight().payload().is_active
                    && !excluded_edges.contains(&edge.weight().id())
                {
                    has_active_edge = true;
                    break;
                }
            }

            if !visited.contains(&neighbor) && has_active_edge {
                queue.push_back(neighbor);
            }
        }

        visited.insert(curr_node);
    }

    visited.len() == graph.node_count()
}
