use std::time::Instant;

use super::state::events_state;
use crate::simulation_controller::serialization_ref_structs::IntoSerializable;
use crate::simulation_controller::{ClientEvent, SCEvent, ServerEvent, SimulationController};

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
    draw_infos_for_selected_nodes, draw_json, draw_modify_topology_section, draw_section_console,
    draw_section_debug, draw_section_graph, draw_section_settings, draw_section_testing,
    draw_toolbar_section,
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
                        println!("");
                    }
                    _ => {}
                }
            });
    }

    fn handle_sc_events(&mut self) {
        for (node_id, channel) in self.simulation_controller.node_event_channels.iter() {
            channel.try_iter().for_each(|e| {
                self.state.events.add_with_limit(
                    SCEvent::new(*node_id, e.into()),
                    MAIN_CONSOLE_SCROLLBACK_LIMIT,
                );
            })
        }

        for (node_id, channel) in self.simulation_controller.client_event_channels.iter() {
            channel.try_iter().for_each(|e| {
                self.state.events.add_with_limit(
                    SCEvent::new(*node_id, e.into()),
                    MAIN_CONSOLE_SCROLLBACK_LIMIT,
                );
            })
        }

        for (node_id, channel) in self.simulation_controller.server_event_channels.iter() {
            channel.try_iter().for_each(|e| {
                self.state.events.add_with_limit(
                    SCEvent::new(*node_id, e.into()),
                    MAIN_CONSOLE_SCROLLBACK_LIMIT,
                );
            })
        }
    }

    // fn random_node_idx(&self) -> Option<NodeIndex> {
    //     let nodes_cnt = self.g.node_count();
    //     if nodes_cnt == 0 {
    //         return None;
    //     }
    //
    //     let random_n_idx = rand::thread_rng().gen_range(0..nodes_cnt);
    //     self.g.g.node_indices().nth(random_n_idx)
    // }
    //
    // fn random_edge_idx(&self) -> Option<EdgeIndex> {
    //     let edges_cnt = self.g.edge_count();
    //     if edges_cnt == 0 {
    //         return None;
    //     }
    //
    //     let random_e_idx = rand::thread_rng().gen_range(0..edges_cnt);
    //     self.g.g.edge_indices().nth(random_e_idx)
    // }
    //
    // fn remove_random_node(&mut self) {
    //     let idx = self.random_node_idx().unwrap();
    //     self.remove_node(idx);
    // }
    //
    // fn add_random_node(&mut self) {
    //     let random_n_idx = self.random_node_idx();
    //     if random_n_idx.is_none() {
    //         return;
    //     }
    //
    //     let random_n = self.g.node(random_n_idx.unwrap()).unwrap();
    //
    //     // location of new node is in in the closest surrounding of random existing node
    //     let mut rng = rand::thread_rng();
    //     let location = Pos2::new(
    //         random_n.location().x + 10. + rng.gen_range(0. ..50.),
    //         random_n.location().y + 10. + rng.gen_range(0. ..50.),
    //     );
    //
    //     let g_idx = self.g.add_node_with_location((), location);
    //
    //     // let sim_node = egui_graphs::Node::new(());
    //     let sim_node_loc = fdg::nalgebra::Point2::new(location.x, location.y);
    //
    //     // let sim_idx = self.sim.add_node((sim_node, sim_node_loc));
    //
    //     // assert_eq!(g_idx, sim_idx);
    // }
    //
    // fn remove_node(&mut self, idx: NodeIndex) {
    //     self.g.remove_node(idx);
    //
    //     // self.sim.remove_node(idx).unwrap();
    //
    //     // update edges count
    //     self.settings_graph.count_edge = self.g.edge_count();
    // }
    //
    // fn add_random_edge(&mut self) {
    //     let random_start = self.random_node_idx().unwrap();
    //     let random_end = self.random_node_idx().unwrap();
    //
    //     self.add_edge(random_start, random_end);
    // }
    //
    // fn add_edge(&mut self, start: NodeIndex, end: NodeIndex) {
    //     self.g.add_edge(start, end, ());
    //
    //     // self.sim.add_edge(start, end, egui_graphs::Edge::new(()));
    // }
    //
    // fn remove_random_edge(&mut self) {
    //     let random_e_idx = self.random_edge_idx();
    //     if random_e_idx.is_none() {
    //         return;
    //     }
    //     let endpoints = self.g.edge_endpoints(random_e_idx.unwrap()).unwrap();
    //
    //     self.remove_edge(endpoints.0, endpoints.1);
    // }
    //
    // fn remove_edge(&mut self, start: NodeIndex, end: NodeIndex) {
    //     let (g_idx, _) = self.g.edges_connecting(start, end).next().unwrap();
    //     self.g.remove_edge(g_idx);
    //
    //     // let sim_idx = self.sim.find_edge(start, end).unwrap();
    //     // self.sim.remove_edge(sim_idx).unwrap();
    // }
    //
    // fn draw_section_simulation(&mut self, ui: &mut Ui) {
    //     ui.horizontal_wrapped(|ui| {
    //         ui.style_mut().spacing.item_spacing = Vec2::new(0., 0.);
    //         ui.label("Force-Directed Simulation is done with ");
    //         ui.hyperlink_to("fdg project", "https://github.com/grantshandy/fdg");
    //     });
    //
    //     ui.separator();
    //
    //     drawers::draw_start_reset_buttons(
    //         ui,
    //         drawers::ValuesConfigButtonsStartReset {
    //             simulation_stopped: self.simulation_stopped,
    //         },
    //         |simulation_stopped: bool, reset_pressed: bool| {
    //             self.simulation_stopped = simulation_stopped;
    //             if reset_pressed {
    //                 self.reset()
    //             };
    //         },
    //     );
    //
    //     ui.add_space(10.);
    //
    //     drawers::draw_simulation_config_sliders(
    //         ui,
    //         drawers::ValuesConfigSlidersSimulation {
    //             dt: self.settings_simulation.dt,
    //             cooloff_factor: self.settings_simulation.cooloff_factor,
    //             scale: self.settings_simulation.scale,
    //         },
    //         |delta_dt: f32, delta_cooloff_factor: f32, delta_scale: f32| {
    //             self.settings_simulation.dt += delta_dt;
    //             self.settings_simulation.cooloff_factor += delta_cooloff_factor;
    //             self.settings_simulation.scale += delta_scale;
    //
    //             // self.force = init_force(&self.settings_simulation);
    //         },
    //     );
    //
    //     ui.add_space(10.);
    //
    //     drawers::draw_counts_sliders(
    //         ui,
    //         drawers::ValuesConfigSlidersGraph {
    //             node_cnt: self.settings_graph.count_node,
    //             edge_cnt: self.settings_graph.count_edge,
    //         },
    //         |delta_nodes, delta_edges| {
    //             self.settings_graph.count_node += delta_nodes as usize;
    //             self.settings_graph.count_edge += delta_edges as usize;
    //
    //             if delta_nodes != 0 {
    //                 if delta_nodes > 0 {
    //                     (0..delta_nodes).for_each(|_| self.add_random_node());
    //                 } else {
    //                     (0..delta_nodes.abs()).for_each(|_| self.remove_random_node());
    //                 }
    //             }
    //
    //             if delta_edges != 0 {
    //                 if delta_edges > 0 {
    //                     (0..delta_edges).for_each(|_| self.add_random_edge());
    //                 } else {
    //                     (0..delta_edges.abs()).for_each(|_| self.remove_random_edge());
    //                 }
    //             }
    //         },
    //     );
    // }

    // fn update_simulation(&mut self) {
    //     if self.simulation_stopped {
    //         return;
    //     }
    //
    //     // self.force.apply(&mut self.sim);
    // }

    //sync locations computed by the simulation with egui_graphs::Graph nodes.
    // fn sync(&mut self) {
    //     self.g.g.node_weights_mut().for_each(|node| {
    //         let sim_computed_point: OPoint<f32, Const<2>> =
    //             self.sim.node_weight(node.id()).unwrap().1;
    //         node.set_location(Pos2::new(
    //             sim_computed_point.coords.x,
    //             sim_computed_point.coords.y,
    //         ));
    //     });
    // }

    // fn reset(&mut self) {
    //     let settings_graph = settings::SettingsGraph::default();
    //     let settings_simulation = settings::SettingsSimulation::default();
    //
    //     let g = Graph::from(&Self::generate_graph());
    //
    //     // let mut force = init_force(&self.settings_simulation);
    //     // let mut sim = fdg::init_force_graph_uniform(g.g.clone(), 1.0);
    //     // force.apply(&mut sim);
    //     // g.g.node_weights_mut().for_each(|node| {
    //     //     let point: fdg::nalgebra::OPoint<f32, fdg::nalgebra::Const<2>> =
    //     //         sim.node_weight(node.id()).unwrap().1;
    //     //     node.set_location(Pos2::new(point.coords.x, point.coords.y));
    //     // });
    //
    //     self.settings_simulation = settings_simulation;
    //     self.settings_graph = settings_graph;
    //
    //     // self.sim = sim;
    //     self.g = g;
    //     // self.force = force;
    // }
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

        Window::new("Events json")
            .collapsible(true)
            .resizable(true)
            .default_width(200.0)
            .default_height(150.0)
            .default_open(true)
            .show(ctx, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    // Load syntax and theme
                    draw_json(ui, &self.state);
                });
            });

        // self.sync();
        // self.update_simulation();
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
