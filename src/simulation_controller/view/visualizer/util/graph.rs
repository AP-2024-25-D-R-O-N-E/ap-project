use egui_graphs::{Edge, Graph, Node};
use petgraph::{
    csr::DefaultIx,
    graph::{EdgeIndex, NodeIndex},
    prelude::StableGraph,
    Undirected,
};
use std::collections::{HashMap, HashSet, VecDeque};
use wg_2024::network::NodeId;

use crate::{
    initializer::drone_vendor::DroneVendor,
    simulation_controller::{
        edge::{CustomEdgeShape, UiEdgePayload},
        node::{CustomNodeShape, UiDroneNode, UiNodePayload, UiNodeType},
        state::{GraphSectionState, State},
        util::*,
        SimulationController,
    },
};

// TODO take into account the disabled edges and crashed drones

pub fn get_payload_mut(graph: &mut UiGraph, node_index: NodeIndex) -> Option<&mut UiNodePayload> {
    match graph.node_mut(node_index) {
        Some(node) => Some(node.payload_mut()),
        None => None,
    }
}

pub fn get_payload(graph: &mut UiGraph, node_index: NodeIndex) -> Option<&UiNodePayload> {
    match graph.node(node_index) {
        Some(node) => Some(node.payload()),
        None => None,
    }
}

pub fn get_drone_node(graph: &mut UiGraph, node_index: NodeIndex) -> &mut UiDroneNode {
    let drone_node = if let UiNodeType::Drone(drone_node) =
        &mut get_payload_mut(graph, node_index).unwrap().node_type
    {
        drone_node
    } else {
        panic!("Unexpected enum variant!")
    };
    drone_node
}

pub fn get_payload_mut_from_state(
    state: &mut State,
    node_index: NodeIndex,
) -> Option<&mut UiNodePayload> {
    get_payload_mut(&mut state.graph_section.g, node_index)
}

pub fn get_payload_from_state(state: &mut State, node_index: NodeIndex) -> Option<&UiNodePayload> {
    get_payload(&mut state.graph_section.g, node_index)
}

pub fn get_drone_node_from_state(state: &mut State, node_index: NodeIndex) -> &mut UiDroneNode {
    get_drone_node(&mut state.graph_section.g, node_index)
}

pub fn check_edge_removal(
    graph: &mut UiGraph,
    status_flag: &mut StatusFlag,
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
            node1_payload.wg_id, node2_payload.wg_id
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
            let neighbors: Vec<NodeIndex> = get_neigbors_with_disabled_edges(&graph.g, node2);
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
            let neighbors: Vec<NodeIndex> = get_neigbors_with_disabled_edges(&graph.g, node1);
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
    // Check that either drone is not crashed
    if get_drone_node(graph, node1).crashed || get_drone_node(graph, node2).crashed {
        *status_flag = Some(Err("Cannot add link to crashed drone".to_string()));
        return false;
    }

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
        let neighbors: Vec<NodeIndex> = get_neigbors_with_disabled_edges(&graph.g, client);
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

pub fn check_drone_addition(
    graph_section_state: &mut GraphSectionState,
    neighbors_wg_ids: &Vec<NodeId>,
) -> Result<(), String> {
    if neighbors_wg_ids.is_empty() {
        return Err("New drone should be connected to at least 1 node".to_string());
    }

    let neighbors_graph_ids: Vec<NodeIndex> = neighbors_wg_ids
        .clone()
        .into_iter()
        .map(|node| *graph_section_state.node_id_map.get(&node).unwrap())
        .collect();

    // Helper function to validate client connection
    fn check_drone_client(graph: &mut UiGraph, client: NodeIndex) -> Result<(), String> {
        let neighbors: Vec<NodeIndex> = get_neigbors_with_disabled_edges(&graph.g, client);
        println!("Neighbors: {:?}", neighbors);
        if neighbors.len() >= 2 {
            Err("Client nodes can have at most 2 neighbors".to_string())
        } else {
            Ok(())
        }
    }

    for node in neighbors_graph_ids {
        let node_payload = match graph_section_state.g.node(node) {
            Some(node) => node.payload().clone(),
            None => {
                return Err(format!(
                    "{:?} is not a valid node id",
                    graph_section_state.g.node(node).unwrap().payload().wg_id
                ));
            }
        };

        match node_payload.node_type {
            UiNodeType::Server(ui_server_node) => (),
            UiNodeType::Client(ui_client_node) => {
                if let Err(err) = check_drone_client(&mut graph_section_state.g, node) {
                    return Err(err);
                }
            }
            UiNodeType::Drone(ui_drone_node) => {
                if ui_drone_node.crashed {
                    return Err(format!(
                        "Cannot add link since drone {:?} is crashed",
                        node_payload.wg_id
                    ));
                }
            }
        };
    }

    Ok(())
}

pub fn add_drone(
    graph_section_state: &mut GraphSectionState,
    neighbors_wg_ids: &Vec<NodeId>,
    wg_id: NodeId,
    pdr: f32,
    drone_vendor: DroneVendor,
    simulation_controller: &mut SimulationController,
) {
    let new_node_graph_index = graph_section_state.g.add_node(UiNodePayload {
        node_type: UiNodeType::Drone(UiDroneNode::new(pdr)),
        vendor: drone_vendor,
        wg_id,
    });

    let neighbors_graph_ids: Vec<NodeIndex> = neighbors_wg_ids
        .clone()
        .into_iter()
        .map(|node| *graph_section_state.node_id_map.get(&node).unwrap())
        .collect();

    for node in neighbors_graph_ids {
        graph_section_state
            .g
            .add_edge(node, new_node_graph_index, UiEdgePayload::default());
    }

    graph_section_state
        .node_id_map
        .insert(wg_id, new_node_graph_index);

    simulation_controller.spawn_drone(wg_id, pdr, neighbors_wg_ids, drone_vendor);
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

pub fn check_node_removal(
    graph: &mut UiGraph,
    status_flag: &mut StatusFlag,
    node: NodeIndex,
) -> bool {
    let neighbors: Vec<NodeIndex> = get_neigbors_with_disabled_edges(&graph.g, node);
    for neighbor in neighbors {
        if !check_edge_removal(graph, status_flag, node, neighbor) {
            return false;
        }
    }
    true
}

pub fn remove_node(
    graph: &mut UiGraph,
    status_flag: &mut StatusFlag,
    node: NodeIndex,
    simulation_controller: &SimulationController,
) {
    let edges: Vec<EdgeIndex> = graph
        .g
        .edges_directed(node, petgraph::Direction::Outgoing)
        .map(|edge| edge.weight().id())
        .collect();
    for edge in edges {
        graph.edge_mut(edge).unwrap().payload_mut().is_active = false;
    }
    get_drone_node(graph, node).crashed = true;
    simulation_controller.send_crash_command(graph.node(node).unwrap().payload().wg_id);
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

pub fn is_connected_without_node_set<E>(
    graph: &StableGraph<
        Edge<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>,
        E,
        Undirected,
    >,
    excluded_edges: HashSet<NodeIndex>,
) -> bool
where
{
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut first_node = None;
    for node in graph.node_indices() {
        if graph.node_weight(node).unwrap().payload().is_active {
            first_node = Some(node);
            break;
        }
    }
    match first_node {
        Some(first_node_unwrapped) => (),
        None => return true,
    }
    queue.push_back(first_node.unwrap());

    while (!queue.is_empty()) {
        let curr_node = queue.pop_front().unwrap();

        if (visited.contains(&curr_node)) {
            continue;
        }

        for neighbor in graph.neighbors_undirected(curr_node) {
            let node_payload = graph.node_weight(neighbor).unwrap().payload();
            if node_payload.is_active && !visited.contains(&neighbor) {
                queue.push_back(neighbor);
            }
        }

        visited.insert(curr_node);
    }

    visited.len() == graph.node_count()
}

pub fn get_neigbors_with_disabled_edges<N>(
    graph: &StableGraph<
        N,
        Edge<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>,
        Undirected,
    >,
    node: NodeIndex,
) -> Vec<NodeIndex> {
    let neighbors: Vec<NodeIndex> = graph.neighbors_undirected(node).collect();
    neighbors
        .into_iter()
        .filter(|neighbor| {
            let mut has_active_edge = false;
            for edge in graph.edges_connecting(node, *neighbor) {
                if edge.weight().payload().is_active {
                    has_active_edge = true;
                    break;
                }
            }
            has_active_edge
        })
        .collect()
}

pub fn is_well_formed(
    graph: &StableGraph<
        Node<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape>,
        Edge<UiNodePayload, UiEdgePayload, Undirected, DefaultIx, CustomNodeShape, CustomEdgeShape>,
        Undirected,
    >,
) -> Result<(), String> {
    // Connected
    if !is_connected_without_edge_set(&graph, HashSet::new()) {
        return Err("Graph is not connected".to_string());
    }

    // Clients have [1,2] neighbors
    // Servers have [2,inf] neighbors
    for node in graph.node_indices() {
        let node_payload = graph.node_weight(node).unwrap().payload();
        if let Err(error) = match &node_payload.node_type {
            UiNodeType::Server(ui_server_node) => {
                let neighbors: Vec<NodeIndex> = get_neigbors_with_disabled_edges(graph, node);
                if neighbors.len() < 2 {
                    Err(format!(
                        "Server {} should have at least 2 neighbors",
                        node_payload.wg_id
                    ))
                } else {
                    Ok(())
                }
            }
            UiNodeType::Client(ui_client_node) => {
                let neighbors: Vec<NodeIndex> = get_neigbors_with_disabled_edges(graph, node);
                if neighbors.len() > 2 {
                    Err(format!(
                        "Client {} should have at most 2 neighbors",
                        node_payload.wg_id
                    ))
                } else {
                    Ok(())
                }
            }
            UiNodeType::Drone(_) => Ok(()),
        } {
            return Err(error);
        }
    }

    Ok(())
}
