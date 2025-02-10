use std::time::Instant;
use std::vec;

use super::state::{events_state, NodeState};
use crate::fragmentation::message::ChatMessage;
use crate::simulation_controller::serialization_ref_structs::IntoSerializable;
use crate::simulation_controller::{
    ClientEvent, SCEvent, SCEventType, ServerEvent, SimulationController,
};

use super::state::EventsState;
use eframe::{run_native, App, CreationContext, NativeOptions};
use egui::text::LayoutJob;
use egui::{Context, ScrollArea, Window};

use egui_extras::syntax_highlighting::CodeTheme;
use egui_graphs::events::Event;

use petgraph::graph::NodeIndex;
use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::as_24_bit_terminal_escaped;
use wg_2024::controller::DroneEvent;

use super::drawers::{
    draw_all_events_as_json, draw_infos_for_selected_nodes, draw_modify_topology_section,
    draw_section_console, draw_section_debug, draw_section_graph, draw_section_settings,
    draw_section_testing, draw_toolbar_section,
};
use super::state::State;

const GRAPH_EVENTS_LIMIT: usize = 100;
const MAIN_CONSOLE_SCROLLBACK_LIMIT: usize = 100;

pub struct SCGui {
    fps: f32,
    last_update_time: Instant,
    frames_last_time_span: usize,
    pan: [f32; 2],
    zoom: f32,
    simulation_controller: SimulationController,
    state: State,
}

impl SCGui {
    fn new(_: &CreationContext<'_>, simulation_controller: SimulationController) -> Self {
        Self {
            fps: 0.,
            last_update_time: Instant::now(),
            frames_last_time_span: 0,

            pan: [0., 0.],
            zoom: 0.,

            state: State::from(&simulation_controller),
            simulation_controller,
        }
    }

    fn update_fps(&mut self) {
        self.frames_last_time_span += 1;
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update_time);
        if elapsed.as_secs() >= 1 {
            self.last_update_time = now;
            self.fps = self.frames_last_time_span as f32 / elapsed.as_secs_f32();
            self.frames_last_time_span = 0;
        }
    }

    fn handle_graph_events(&mut self) {
        self.state
            .graph_section
            .graph_event_consumer
            .try_iter()
            .for_each(|e| {
                if self.state.debug_section.graph_events.len() > GRAPH_EVENTS_LIMIT {
                    self.state.debug_section.graph_events.remove(0);
                }
                self.state
                    .debug_section
                    .graph_events
                    .push(serde_json::to_string(&e).unwrap());

                match e {
                    Event::Pan(payload) => self.pan = payload.new_pan,
                    Event::Zoom(payload) => self.zoom = payload.new_zoom,
                    Event::NodeDoubleClick(double_click) => {
                        let node_index = NodeIndex::new(double_click.id);
                        self.state
                            .graph_section
                            .g
                            .node_mut(node_index)
                            .unwrap()
                            .set_selected(true);

                        for node in self.state.graph_section.g.selected_nodes() {
                            print!("{:?} ", node);
                            self.state.node_info_section.open_window(*node);
                        }
                        self.state.node_info_section.open_window(node_index);
                        println!();
                    }
                    _ => {}
                }
            });
    }

    fn handle_sc_shortcut(&self, event: &SCEvent) {
        if self.state.toolbar_section.handle_shortcuts {
            if let SCEventType::Drone(DroneEvent::ControllerShortcut(packet)) = &event.event_type {
                self.simulation_controller
                    .handle_sc_shortcut(packet.clone());
            }
        }
    }

    fn handle_available_peers_update(&mut self, event: &SCEvent) {
        if let SCEventType::Client(ClientEvent::ResponseClientsReceived(ids)) = &event.event_type {
            let sender_node_index = self
                .state
                .graph_section
                .node_id_map
                .get(&event.sender_id)
                .unwrap();
            if let NodeState::Client(client_state) = self
                .state
                .node_info_section
                .states
                .get_mut(sender_node_index)
                .unwrap()
            {
                client_state.available_peers = ids.to_vec();
                for peers in &client_state.available_peers {
                    client_state.chat_histories.insert(*peers, vec![]);
                }
            }
        }
    }

    fn handle_available_chat_hystory_update(&mut self, event: &SCEvent) {
        if let SCEventType::Client(ClientEvent::ResponseHistoryReceived { partner, history }) =
            &event.event_type
        {
            let sender_node_index = self
                .state
                .graph_section
                .node_id_map
                .get(&event.sender_id)
                .unwrap();

            if let NodeState::Client(client_state) = self
                .state
                .node_info_section
                .states
                .get_mut(sender_node_index)
                .unwrap()
            {
                println!("{:?}", history);
                client_state
                    .chat_histories
                    .insert(*partner, history.to_vec());
            }
        } else if let SCEventType::Client(ClientEvent::TextMessage { from, to, text }) =
            &event.event_type
        {
            let sender_node_index = self
                .state
                .graph_section
                .node_id_map
                .get(&event.sender_id)
                .unwrap();

            if let NodeState::Client(client_state) = self
                .state
                .node_info_section
                .states
                .get_mut(sender_node_index)
                .unwrap()
            {
                let chat_msg = ChatMessage::TextMessage {
                    from: *from,
                    to: *to,
                    text: text.to_string(),
                };

                if let Some(chat_history) = client_state.chat_histories.get_mut(from) {
                    chat_history.push(chat_msg.clone());
                }
            }
        } else if let SCEventType::Client(ClientEvent::FileMessage {
            from,
            to,
            file_path,
        }) = &event.event_type
        {
            let sender_node_index = self
                .state
                .graph_section
                .node_id_map
                .get(&event.sender_id)
                .unwrap();

            if let NodeState::Client(client_state) = self
                .state
                .node_info_section
                .states
                .get_mut(sender_node_index)
                .unwrap()
            {
                let chat_msg = ChatMessage::FileMessage {
                    from: *from,
                    to: *to,
                    file_path: file_path.clone(),
                };

                if let Some(chat_history) = client_state.chat_histories.get_mut(from) {
                    chat_history.push(chat_msg.clone());
                }
            }
        } else if let SCEventType::Client(ClientEvent::CreatedFileLocal {
            from,
            to,
            file_path,
        }) = &event.event_type
        {
            let sender_node_index = self
                .state
                .graph_section
                .node_id_map
                .get(&event.sender_id)
                .unwrap();

            if let NodeState::Client(client_state) = self
                .state
                .node_info_section
                .states
                .get_mut(sender_node_index)
                .unwrap()
            {
                let chat_msg = ChatMessage::FileMessage {
                    from: *from,
                    to: *to,
                    file_path: file_path.clone(),
                };

                if let Some(chat_history) = client_state.chat_histories.get_mut(to) {
                    chat_history.push(chat_msg.clone());
                }
            }
        }
    }

    fn handle_sc_events(&mut self) {
        let mut events = vec![];
        for (node_id, channel) in self.simulation_controller.node_event_channels.iter() {
            let curr_events: Vec<SCEvent> = channel
                .try_iter()
                .map(|e| SCEvent::new(*node_id, e.into()))
                .collect();

            events.extend_from_slice(&curr_events);
        }

        for (node_id, channel) in self.simulation_controller.client_event_channels.iter() {
            let curr_events: Vec<SCEvent> = channel
                .try_iter()
                .map(|e| SCEvent::new(*node_id, e.into()))
                .collect();

            events.extend_from_slice(&curr_events);
        }

        for (node_id, channel) in self.simulation_controller.server_event_channels.iter() {
            let curr_events: Vec<SCEvent> = channel
                .try_iter()
                .map(|e| SCEvent::new(*node_id, e.into()))
                .collect();
            events.extend_from_slice(&curr_events);
        }

        for sc_event in events {
            self.handle_sc_shortcut(&sc_event);
            self.handle_available_peers_update(&sc_event);
            self.handle_available_chat_hystory_update(&sc_event);
            self.state
                .events
                .add_with_limit(sc_event, MAIN_CONSOLE_SCROLLBACK_LIMIT);
        }
    }
}

impl App for SCGui {
    fn update(&mut self, ctx: &Context, _: &mut eframe::Frame) {
        let custom_frame = egui::Frame {
            fill: egui::Color32::from_rgb(10, 10, 10), // Custom background color
            stroke: egui::Stroke::NONE,
            rounding: egui::Rounding::ZERO,
            inner_margin: egui::Margin::same(10.0),
            outer_margin: egui::Margin::ZERO,
            ..Default::default()
        };

        egui::TopBottomPanel::top("top_panel")
            .frame(custom_frame)
            .show(ctx, |ui| draw_toolbar_section(ui, &mut self.state));

        if self.state.toolbar_section.console_open {
            egui::TopBottomPanel::bottom("bottom_panel")
                .frame(custom_frame)
                .resizable(true)
                .show(ctx, |ui| draw_section_console(ui, &mut self.state));
        }

        egui::CentralPanel::default().show(ctx, |ui| draw_section_graph(ui, &mut self.state));

        if self.state.toolbar_section.settings_open {
            Window::new("Settings")
                .collapsible(true)
                .resizable(true)
                .default_width(200.0)
                .default_height(100.0)
                .show(ctx, |ui| {
                    ScrollArea::vertical()
                        .show(ui, |ui| draw_section_settings(ui, &mut self.state));
                });
        }

        if self.state.toolbar_section.debug_open {
            Window::new("Debug")
                .collapsible(true)
                .resizable(true)
                .default_width(200.0)
                .default_height(100.0)
                .default_open(true)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        draw_section_debug(ui, &mut self.state);
                    });
                });
        }

        if self.state.toolbar_section.test_open {
            Window::new("Test")
                .collapsible(true)
                .resizable(true)
                .default_width(300.0)
                .default_height(150.0)
                .default_open(true)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        draw_section_testing(ui, &mut self.state, &self.simulation_controller);
                    });
                });
        }

        if self.state.toolbar_section.modify_topology_open {
            Window::new("Modify topology")
                .collapsible(true)
                .resizable(true)
                .default_width(200.0)
                .default_height(150.0)
                .default_open(true)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        draw_modify_topology_section(
                            ui,
                            &mut self.state,
                            &mut self.simulation_controller,
                        );
                    });
                });
        }

        if self.state.toolbar_section.events_json_open {
            Window::new("Events json")
                .collapsible(true)
                .resizable(true)
                .default_width(200.0)
                .default_height(150.0)
                .default_open(true)
                .show(ctx, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        // Load syntax and theme
                        draw_all_events_as_json(ui, &self.state);
                    });
                });
        }

        // self.sync();
        // self.update_simulation();
        self.state.file_dialog.update(ctx);
        draw_infos_for_selected_nodes(ctx, &mut self.state, &self.simulation_controller);
        self.handle_graph_events();
        self.handle_sc_events();
        self.update_fps();
    }
}

pub fn run_gui(title: String, simulation_controller: SimulationController) {
    run_native(
        &title,
        NativeOptions::default(),
        Box::new(|cc| {
            let x = SCGui::new(cc, simulation_controller);
            Ok(Box::new(x))
        }),
    )
    .unwrap();
}
