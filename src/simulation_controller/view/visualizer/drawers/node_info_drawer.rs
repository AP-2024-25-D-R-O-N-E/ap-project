use egui::{CollapsingHeader, Context, ScrollArea, Ui, Window};
use petgraph::graph::NodeIndex;

use crate::simulation_controller::state::{DisplayOptions, State};

pub fn draw_infos_for_selected_nodes(ctx: &Context, state: &mut State) {
    for node_index in state.node_info_section.opened_windows.clone().iter() {
        draw_node_info(ctx, *node_index, state);
    }
}
pub fn draw_node_info(ctx: &Context, node_index: NodeIndex, state: &mut State) {
    let node_payload = state.graph_section.g.node(node_index).unwrap().payload();
    Window::new(format!("Node {}", node_payload.wg_id))
        .collapsible(true)
        .resizable(true)
        .default_width(200.0)
        .default_height(100.0)
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("my_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        let payload = state.graph_section.g.node(node_index).unwrap().payload();
                        ui.label("Node type".to_string());
                        ui.label(payload.get_type().to_string());
                        ui.end_row();

                        ui.label("Vendor".to_string());
                        ui.label(payload.vendor.to_string());
                        ui.end_row();

                        if ui.button("Close").clicked() {
                            let n = state
                                .graph_section
                                .g
                                .node_mut(node_index)
                                .unwrap()
                                .set_selected(false);
                        }
                        if ui.button("Close others").clicked() {
                            for selected_node_index in
                                state.graph_section.g.selected_nodes().to_owned()
                            {
                                if selected_node_index != node_index {
                                    state
                                        .graph_section
                                        .g
                                        .node_mut(selected_node_index)
                                        .unwrap()
                                        .set_selected(false);
                                }
                            }
                        }
                        ui.end_row();
                    });
                CollapsingHeader::new("Logs").show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        for x in state.events.get_events_list(DisplayOptions::from_index(
                            state.graph_section.wg_id(node_index).unwrap(),
                        )) {
                            ui.label(
                                egui::RichText::new(format!("{:?}", x))
                                    .color(egui::Color32::LIGHT_GRAY),
                            );
                        }
                    })
                });
                // ui.horizontal(|ui| {})
            });
        });
}
