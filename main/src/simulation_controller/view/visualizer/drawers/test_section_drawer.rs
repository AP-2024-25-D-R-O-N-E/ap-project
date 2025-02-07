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
    str::FromStr,
};
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{FloodRequest, NodeType, Packet, PacketType},
};

use super::super::util::graph::*;

use crate::simulation_controller::{
    edge::{CustomEdgeShape, UiEdgePayload},
    node::{CustomNodeShape, UiNodePayload, UiNodeType},
    state::{AckType, State},
    util::*,
    SimulationController,
};

pub fn draw_section_testing(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
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
        });

    CollapsingHeader::new("Send ack/nack")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    send_ack_nack_section(ui, state, simulation_controller)
                });
        });

    CollapsingHeader::new("Send flood request")
        .default_open(true)
        .show(ui, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    send_flood_req_section(ui, state, simulation_controller)
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
            egui::TextEdit::singleline(&mut state.test_section.msg_fragment_routing_path_string)
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
            match get_path_between_selected_nodes(state) {
                Ok(rv) => state.test_section.msg_fragment_routing_path_string = rv,
                Err(err) => state.test_section.packet_sender_status_flag = Some(Err(err)),
            }
        }

        if ui
            .button("Invert")
            .on_hover_text("Invert the current path")
            .clicked()
        {
            let mut reversed_path = String::new();
            for c in state
                .test_section
                .msg_fragment_routing_path_string
                .chars()
                .rev()
            {
                reversed_path.push(c);
            }
            state.test_section.msg_fragment_routing_path_string = reversed_path;
        }
    });

    // Send Button
    if ui
        .add_sized(ui.available_size(), egui::Button::new("Send"))
        .clicked()
    {
        match (
            parse_data(state.test_section.msg_fragment_routing_path_string.clone()),
            parse_data(state.test_section.msg_frag_data_string.clone()),
        ) {
            (Ok(mut parsed_path_vec), Ok(mut parsed_data_vec)) => {
                if parsed_data_vec.len() > 128 {
                    state.test_section.packet_sender_status_flag =
                        Some(Err("Data should be at most 128 chars long".to_string()));
                    return;
                } else {
                    parsed_data_vec.resize(128, 0);
                    let mut parsed_data: [u8; 128] = [0; 128];
                    match parsed_data_vec.try_into() {
                        Ok(v) => {
                            parsed_data = v;
                            let mut packet = simulation_controller.default_msg_fragment.clone();
                            packet.routing_header.hops = parsed_path_vec;
                            match &mut packet.pack_type {
                                PacketType::MsgFragment(fragment) => {
                                    fragment.data = parsed_data;
                                }
                                _ => (),
                            }
                            simulation_controller.send_msg_fragment(packet);
                            state.test_section.packet_sender_status_flag =
                                Some(Ok("Msg fragment sent".to_string()))
                        }
                        Err(_) => (),
                    }
                }
            }
            (Err(error), _) => state.test_section.packet_sender_status_flag = error,
            (_, Err(error)) => state.test_section.packet_sender_status_flag = error,
        };
    }

    ui.end_row();
    display_status_flag(&state.test_section.packet_sender_status_flag, ui);
    ui.end_row();
}

pub fn send_ack_nack_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    ui.label("Routing path");
    if ui
        .add_sized(
            ui.available_size(),
            egui::TextEdit::singleline(&mut state.test_section.ack_nack_routing_path_string)
                .hint_text("Enter node IDs separated by commas"), // .tooltip_text("The list of node id separated by a comma"),
        )
        .changed()
    {}

    ui.end_row();

    ui.horizontal(|ui| {
        if ui
            .button("Get from selection")
            .on_hover_text("Get a random path between the two selected nodes")
            .clicked()
        {
            match get_path_between_selected_nodes(state) {
                Ok(rv) => state.test_section.ack_nack_routing_path_string = rv,
                Err(err) => state.test_section.ack_nack_sender_status_flag = Some(Err(err)),
            }
        }

        if ui
            .button("Invert")
            .on_hover_text("Invert the current path")
            .clicked()
        {
            state.test_section.ack_nack_routing_path_string = state
                .test_section
                .ack_nack_routing_path_string
                .chars()
                .rev()
                .collect();
        }
    });

    ui.horizontal(|ui| {
        ui.radio_value(&mut state.test_section.ack_type, AckType::Ack, "Ack");
        ui.radio_value(&mut state.test_section.ack_type, AckType::Nack, "Nack");
        if ui
            .add_sized(ui.available_size(), egui::Button::new("Send"))
            .clicked()
        {
            match parse_data(state.test_section.ack_nack_routing_path_string.clone()) {
                Ok(parsed_path) => match state.test_section.ack_type {
                    AckType::Ack => {
                        let mut packet = simulation_controller.default_ack.clone();
                        packet.routing_header.hops = parsed_path;
                        simulation_controller.send_ack(packet);
                    }
                    AckType::Nack => {
                        let mut packet = simulation_controller.default_nack.clone();
                        packet.routing_header.hops = parsed_path;
                        simulation_controller.send_nack(packet);
                    }
                },
                Err(error) => state.test_section.ack_nack_sender_status_flag = error,
            }
        }
    });
    ui.end_row();

    display_status_flag(&state.test_section.ack_nack_sender_status_flag, ui);
    ui.end_row();
    // match parse_path
    // let mut packet = simulation_controller.default_ack.clone();
}

pub fn send_flood_req_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    ui.horizontal(|ui| {
        ui.label("Initiator id");
        ui.add(egui::DragValue::new(
            &mut state.test_section.flood_req_initiator_id,
        ));
    });

    if ui
        .add_sized(ui.available_size(), egui::Button::new("Send"))
        .clicked()
    {
        // flood_req.initiator_id = state.test_section.flood_req_initiator_id;
        let curr_node_wg_id = state.test_section.flood_req_initiator_id;
        match state.graph_section.node_id_map.get(&curr_node_wg_id) {
            Some(node) => {
                if is_client(&state.graph_section.g.g, *node)
                    || is_server(&state.graph_section.g.g, *node)
                {
                    for neighbor in state.graph_section.g.g.neighbors(*node) {
                        let neighbor_wg_id = state
                            .graph_section
                            .g
                            .node(neighbor)
                            .unwrap()
                            .payload()
                            .wg_id;

                        let mut flood_req = FloodRequest::new(
                            state.test_section.flood_latest_used_id,
                            state.test_section.flood_req_initiator_id,
                        );
                        match &state
                            .graph_section
                            .g
                            .node(*node)
                            .unwrap()
                            .payload()
                            .node_type
                        {
                            UiNodeType::Server(ui_server_node) => {
                                flood_req.path_trace = vec![(curr_node_wg_id, NodeType::Server)]
                            }
                            UiNodeType::Client(ui_client_node) => {
                                flood_req.path_trace = vec![(curr_node_wg_id, NodeType::Client)]
                            }
                            UiNodeType::Drone(ui_drone_node) => {
                                flood_req.path_trace = vec![(curr_node_wg_id, NodeType::Drone)]
                            }
                        }

                        let routing_header =
                            SourceRoutingHeader::new(vec![curr_node_wg_id, neighbor_wg_id], 1);
                        let packet = Packet::new_flood_request(routing_header, 0, flood_req);
                        // packet.routing_header.hops = vec![flood_req.initiator_id, neighbor_wg_id];
                        // println!("{:?}", packet);
                        simulation_controller.send_flood_request(packet.clone(), neighbor_wg_id);
                    }

                    state.test_section.flood_latest_used_id += 1;
                } else {
                    state.test_section.flood_req_sender_status_flag = Some(Err(
                        "The initiator id should be a client or a server".to_string(),
                    ))
                }
            }
            None => {
                state.test_section.flood_req_sender_status_flag =
                    Some(Err("Inalid node id".to_string()))
            }
        }
    }

    ui.end_row();
    display_status_flag(&state.test_section.flood_req_sender_status_flag, ui);
}

fn parse_data<T>(path: String) -> Result<Vec<T>, StatusFlag>
where
    T: FromStr,
{
    for c in path.chars() {
        if !(c.is_digit(10) || c == ',' || c.is_whitespace()) {
            return Err(Some(Err("The path is malformed".to_string())));
        }
    }

    for c in path.chars() {
        if !(c.is_digit(10) || c == ',' || c.is_whitespace()) {
            return Err(Some(Err("The path is malformed".to_string())));
        }
    }

    let mut ok = true;
    let mut parsed_data_vec: Vec<T> = path
        .split(',')
        .filter_map(|s| {
            let rv = s.trim().parse::<T>().ok();
            match rv {
                None => ok = false,
                _ => (),
            }

            rv
        })
        .collect();
    match ok {
        true => Ok(parsed_data_vec),
        false => return Err(Some(Err("The path is malformed".to_string()))),
    }
}

fn get_path_between_selected_nodes(state: &mut State) -> Result<String, String> {
    if state.graph_section.g.selected_nodes().len() != 2 {
        Err("Please select exactly 2 nodes".to_string())
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
                    let actual_node_index = state.graph_section.g.node(n).unwrap().payload().wg_id;
                    new_path.push_str(format!("{}, ", actual_node_index).as_str());
                }
                if new_path.len() != 0 {
                    new_path.truncate(new_path.len() - 2);
                }
                Ok(new_path)
            }
            None => Err("Cannot find path. Is the graph connected?".to_string()),
        }
    }
}

fn display_status_flag(status_flag: &StatusFlag, ui: &mut Ui) {
    match &status_flag {
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
