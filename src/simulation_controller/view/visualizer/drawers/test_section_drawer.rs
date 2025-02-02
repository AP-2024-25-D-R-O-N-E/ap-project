use egui::{CollapsingHeader, Color32, RichText, Ui};
use egui_graphs::{Edge, Node};
use petgraph::{
    algo::{self, connected_components, dijkstra::dijkstra},
    csr::DefaultIx,
    graph::{EdgeIndex, NodeIndex},
    prelude::StableGraph,
    visit::{IntoEdges, Visitable},
    Undirected,
};
use std::{
    collections::{HashSet, VecDeque},
    hash::Hash,
};
use wg_2024::network::NodeId;

use crate::simulation_controller::{
    edge::{CustomEdgeShape, UiEdgePayload},
    node::{CustomNodeShape, UiNodePayload, UiNodeType},
    state::State,
    util::*,
    SimulationController,
};

pub fn draw_section_testing(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    // CollapsingHeader::new("Last Events")
    //     .default_open(false)
    //     .show(ui, |ui| {
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default fragment").clicked() {
    //                 simulation_controller
    //                     .send_default_msg_fragment(state.test_section.send_default_fragment_node_id)
    //             }
    //             ui.add(
    //                 egui::DragValue::new(&mut state.test_section.send_default_fragment_node_id)
    //                     .speed(0.1),
    //             );
    //         });
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default flood request").clicked() {
    //                 simulation_controller.send_default_flood_request(
    //                     state.test_section.send_default_flood_request_node_id,
    //                 )
    //             }
    //             ui.add(
    //                 egui::DragValue::new(
    //                     &mut state.test_section.send_default_flood_request_node_id,
    //                 )
    //                 .speed(0.1),
    //             );
    //         });
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default ack").clicked() {
    //                 simulation_controller
    //                     .send_default_ack(state.test_section.send_default_ack_node_id)
    //             }
    //             ui.add(
    //                 egui::DragValue::new(&mut state.test_section.send_default_ack_node_id)
    //                     .speed(0.1),
    //             );
    //         });
    //         ui.horizontal(|ui| {
    //             if ui.button("Send default nack").clicked() {
    //                 simulation_controller
    //                     .send_default_nack(state.test_section.send_default_nack_node_id)
    //             }
    //             ui.add(
    //                 egui::DragValue::new(&mut state.test_section.send_default_nack_node_id)
    //                     .speed(0.1),
    //             );
    //         });
    //     });

    CollapsingHeader::new("Send msg fragment")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    send_msg_fragment_section(ui, state, simulation_controller)
                });

            //     ui.horizontal(|ui| {
            //         if ui.button("Send default fragment").clicked() {
            //             simulation_controller
            //                 .send_default_msg_fragment(state.test_section.send_default_fragment_node_id)
            //         }
            //         ui.add(
            //             egui::DragValue::new(&mut state.test_section.send_default_fragment_node_id)
            //                 .speed(0.1),
            //         );
            //     });
            //     ui.horizontal(|ui| {
            //         if ui.button("Send default flood request").clicked() {
            //             simulation_controller.send_default_flood_request(
            //                 state.test_section.send_default_flood_request_node_id,
            //             )
            //         }
            //         ui.add(
            //             egui::DragValue::new(
            //                 &mut state.test_section.send_default_flood_request_node_id,
            //             )
            //             .speed(0.1),
            //         );
            //     });
            //     ui.horizontal(|ui| {
            //         if ui.button("Send default ack").clicked() {
            //             simulation_controller
            //                 .send_default_ack(state.test_section.send_default_ack_node_id)
            //         }
            //         ui.add(
            //             egui::DragValue::new(&mut state.test_section.send_default_ack_node_id)
            //                 .speed(0.1),
            //         );
            //     });
            //     ui.horizontal(|ui| {
            //         if ui.button("Send default nack").clicked() {
            //             simulation_controller
            //                 .send_default_nack(state.test_section.send_default_nack_node_id)
            //         }
            //         ui.add(
            //             egui::DragValue::new(&mut state.test_section.send_default_nack_node_id)
            //                 .speed(0.1),
            //         );
            //     });
        });

    CollapsingHeader::new("Add/remove sender")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    add_remove_sender_section(ui, state, simulation_controller)
                });
        });
}

pub fn send_msg_fragment_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    // Routing Path Input
    ui.label("Routing path");
    if ui
        .add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(&mut state.test_section.routing_path_string)
                .hint_text("Enter node IDs separated by commas"), // .tooltip_text("The list of node id separated by a comma"),
        )
        .changed()
    {}

    ui.end_row();

    // Msg data input
    ui.label("Msg data");
    if ui
        .add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(&mut state.test_section.msg_frag_data_string)
                .hint_text("The msg_fragment payload. Just input a stream of max 128 characters"), // .tooltip_text(),
        )
        .changed()
    {}
    ui.end_row();

    // Get from nodes button
    ui.horizontal(|ui| {
        if ui
            .button("Get from selection")
            .on_hover_text("Get a random path between the two selected nodes")
            .clicked()
        {
            if state.graph_section.g.selected_nodes().len() != 2 {
                state.test_section.packet_sender_status_flag =
                    Some(Err("Please select exactly 2 nodes".to_string()));
            } else {
                state.test_section.packet_sender_status_flag = None;
                let start_node = state.graph_section.g.selected_nodes()[0].clone();
                let end_node = state.graph_section.g.selected_nodes()[1].clone();
                let g = &*state.graph_section.g.g();

                let path = algo::astar(g, start_node, |n| n == end_node, |e| 1, |_| 0);
                match path {
                    Some((_, path)) => {
                        let mut new_path = String::new();
                        for n in path {
                            let actual_node_index =
                                state.graph_section.g.node(n).unwrap().payload().wg_id;
                            new_path.push_str(format!("{}, ", actual_node_index).as_str());
                        }
                        if new_path.len() != 0 {
                            new_path.truncate(new_path.len() - 2);
                        }
                        state.test_section.routing_path_string = new_path
                    }
                    None => todo!(),
                }
            }
        }

        if ui
            .button("Invert")
            .on_hover_text("Invert the current path")
            .clicked()
        {
            let mut reversed_path = String::new();
            for c in state.test_section.routing_path_string.chars().rev() {
                reversed_path.push(c);
            }
            state.test_section.routing_path_string = reversed_path;
        }
    });

    // Send Button
    if ui
        .add_sized(ui.available_size(), egui::Button::new("Send"))
        .clicked()
    {
        let mut ok = true;
        for c in state.test_section.routing_path_string.chars() {
            if !(c.is_digit(10) || c == ',' || c.is_whitespace()) {
                state.test_section.packet_sender_status_flag =
                    Some(Err("The path is malformed".to_string()));
                ok = false;
                break;
            }
        }

        for c in state.test_section.msg_frag_data_string.chars() {
            if !(c.is_digit(10) || c == ',' || c.is_whitespace()) {
                state.test_section.packet_sender_status_flag =
                    Some(Err("The data is malformed".to_string()));
                ok = false;
                break;
            }
        }
        let mut parsed_data_vec: Vec<u8> = state
            .test_section
            .msg_frag_data_string
            .split(',')
            .filter_map(|s| {
                let rv = s.trim().parse::<u8>().ok();
                match rv {
                    None => {
                        state.test_section.packet_sender_status_flag =
                            Some(Err("The data is malformed".to_string()));
                        ok = false;
                    }
                    _ => (),
                }

                rv
            })
            .collect();

        if parsed_data_vec.len() > 128 {
            state.test_section.packet_sender_status_flag =
                Some(Err("Data should be at most 128 chars long".to_string()));
            ok = false;
        }
        parsed_data_vec.resize(128, 0);

        if ok {
            let parsed_path: Vec<NodeId> = state
                .test_section
                .routing_path_string
                .split(',')
                .filter_map(|s| s.trim().parse::<NodeId>().ok())
                .collect();
            let mut parsed_data: [u8; 128] = [0; 128];
            match parsed_data_vec.try_into() {
                Ok(v) => {
                    parsed_data = v;
                }
                Err(_) => (),
            }

            let mut packet = simulation_controller.default_msg_fragment.clone();
            packet.routing_header.hops = parsed_path;
            match &mut packet.pack_type {
                wg_2024::packet::PacketType::MsgFragment(fragment) => {
                    fragment.data = parsed_data;
                }
                _ => (),
            }
            simulation_controller.send_msg_fragment(packet);
        }
    }

    ui.end_row();
    match &state.test_section.packet_sender_status_flag {
        Some(status) => match status {
            Ok(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_GREEN));
            }
            Err(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_RED));
            }
        },
        None => (),
    }
    ui.end_row();
}

pub fn add_remove_sender_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    let check_parameters = |state: &mut State| {
        if state.graph_section.g.selected_nodes().len() != 2 {
            state.test_section.channel_modifier_status_flag =
                Some(Err("Please select exactly 2 nodes".to_string()));
            false
        } else {
            state.test_section.channel_modifier_status_flag = None;
            true
        }
    };

    if ui.button("Add sender").clicked() {
        if check_parameters(state) {
            let node1 = state.graph_section.g.selected_nodes()[0];
            let node2 = state.graph_section.g.selected_nodes()[1];

            if check_edge_addition(state, node1, node2) {
                add_edge_between(state, node1, node2, simulation_controller);
            }
        }
    }

    if (ui.button("Remove sender").clicked()) {
        if check_parameters(state) {
            let node1 = state.graph_section.g.selected_nodes()[0];
            let node2 = state.graph_section.g.selected_nodes()[1];

            if (check_edge_removal(state, node1, node2)) {
                remove_edges_between(state, node1, node2, simulation_controller);
            }
        }
    }

    ui.end_row();

    match &state.test_section.channel_modifier_status_flag {
        Some(status) => match status {
            Ok(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_GREEN));
            }
            Err(s) => {
                ui.label(RichText::new(s).color(colors::MUTED_RED));
            }
        },
        None => (),
    }
}

fn check_edge_removal(state: &mut State, node1: NodeIndex, node2: NodeIndex) -> bool {
    let g = &state.graph_section.g.g;
    let mut edges = HashSet::new();
    for (edge_index, edge) in state.graph_section.g.edges_connecting(node1, node2) {
        edges.insert(edge_index);
    }

    let node1_payload = state.graph_section.g.node(node1).unwrap().payload().clone();
    let node2_payload = state.graph_section.g.node(node2).unwrap().payload().clone();

    // Check if an edge exists between the two nodes
    if edges.is_empty() {
        state.test_section.channel_modifier_status_flag = Some(Err(format!(
            "No channel between {} and {}",
            node1_payload.wg_id, node1_payload.wg_id
        )));
        return false;
    }
    // Check if removing this edge would disconnect the graph
    else if !is_connected_without_edge_set(g, edges) {
        state.test_section.channel_modifier_status_flag = Some(Err(
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
            let neighbors: Vec<NodeIndex> = g.neighbors_undirected(node2).collect();
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
            let neighbors: Vec<NodeIndex> = g.neighbors_undirected(node1).collect();
            if neighbors.len() == 2 {
                false
            } else {
                true
            }
        }
        _ => true,
    } {
        state.test_section.channel_modifier_status_flag = Some(Err(
            "Cannot remove edge: each server should be connected to at least 2 nodes".to_string(),
        ));
        return false;
    }

    true
}

fn remove_edges_between(
    state: &mut State,
    node1: NodeIndex,
    node2: NodeIndex,
    simulation_controller: &SimulationController,
) {
    state.graph_section.g.remove_edges_between(node1, node2);
    state.test_section.channel_modifier_status_flag =
        Some(Ok("Sender removed with success".to_string()));

    let node1_wg_id = state.graph_section.g.node(node1).unwrap().payload().wg_id;
    let node2_wg_id = state.graph_section.g.node(node2).unwrap().payload().wg_id;

    simulation_controller.send_remove_sender_command(node1_wg_id, node2_wg_id);
    simulation_controller.send_remove_sender_command(node2_wg_id, node1_wg_id);
}

fn check_edge_addition(state: &mut State, node1: NodeIndex, node2: NodeIndex) -> bool {
    let g = &state.graph_section.g.g;

    // Ensure channel does not already exist
    let edges: HashSet<_> = state
        .graph_section
        .g
        .edges_connecting(node1, node2)
        .map(|(edge_index, _)| edge_index)
        .collect();
    if edges.len() > 0 {
        state.test_section.channel_modifier_status_flag = Some(Err(
            "Channel already exists between the two nodes".to_string(),
        ));
        return false;
    }

    // Helper function to validate client connection
    fn check_drone_client(state: &mut State, client: NodeIndex) -> bool {
        let neighbors: Vec<NodeIndex> = state
            .graph_section
            .g
            .g
            .neighbors_undirected(client)
            .collect();
        if neighbors.len() >= 2 {
            state.test_section.channel_modifier_status_flag =
                Some(Err("Client nodes can have at most 2 neighbors".to_string()));
            return false;
        }
        true
    }

    // Helper function to handle non-drone connections
    fn check_non_drone(state: &mut State) -> bool {
        state.test_section.channel_modifier_status_flag =
            Some(Err("Cannot have link between two non-drones".to_string()));
        false
    }

    // Check client contraints
    let node1_payload = match state.graph_section.g.node(node1) {
        Some(node) => node.payload().clone(),
        None => return false, // Early return if node1 is invalid
    };

    let node2_payload = match state.graph_section.g.node(node2) {
        Some(node) => node.payload().clone(),
        None => return false, // Early return if node2 is invalid
    };
    match (node1_payload.node_type, node2_payload.node_type) {
        (
            crate::simulation_controller::node::UiNodeType::Drone(_),
            crate::simulation_controller::node::UiNodeType::Client(_),
        ) => check_drone_client(state, node2),
        (
            crate::simulation_controller::node::UiNodeType::Client(_),
            crate::simulation_controller::node::UiNodeType::Drone(_),
        ) => check_drone_client(state, node1),
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
        ) => check_non_drone(state),
        _ => true,
    }
}

fn add_edge_between(
    state: &mut State,
    node1: NodeIndex,
    node2: NodeIndex,
    simulation_controller: &SimulationController,
) {
    state
        .graph_section
        .g
        .add_edge(node1, node2, UiEdgePayload::default());
    state.test_section.channel_modifier_status_flag =
        Some(Ok("Sender added with success".to_string()));
    let node1_wg_id = state.graph_section.g.node(node1).unwrap().payload().wg_id;
    let node2_wg_id = state.graph_section.g.node(node1).unwrap().payload().wg_id;

    simulation_controller.send_add_sender_command(node1_wg_id, node2_wg_id);
    simulation_controller.send_add_sender_command(node2_wg_id, node1_wg_id);
}

fn bfs_with_disabled_edges<N>(
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

fn is_connected_without_edge_set<N>(
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
