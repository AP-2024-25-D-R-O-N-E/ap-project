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
use wg_2024::network::NodeId;

use super::super::util::graph::*;

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
                    send_flood_req_nack_section(ui, state, simulation_controller)
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
                                wg_2024::packet::PacketType::MsgFragment(fragment) => {
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

    if ui.button("Send").clicked() {
        match parse_data(state.test_section.ack_nack_routing_path_string.clone()) {
            Ok(parsed_path) => {
                let mut packet = simulation_controller.default_ack.clone();
                packet.routing_header.hops = parsed_path;
                simulation_controller.send_ack(packet);
            }
            Err(error) => state.test_section.ack_nack_sender_status_flag = error,
        }
    }
    ui.end_row();

    match &state.test_section.ack_nack_sender_status_flag {
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
    // match parse_path
    // let mut packet = simulation_controller.default_ack.clone();
}

pub fn send_flood_req_nack_section(
    ui: &mut Ui,
    state: &mut State,
    simulation_controller: &SimulationController,
) {
    ui.end_row();
    // match parse_path
    // let mut packet = simulation_controller.default_ack.clone();
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
