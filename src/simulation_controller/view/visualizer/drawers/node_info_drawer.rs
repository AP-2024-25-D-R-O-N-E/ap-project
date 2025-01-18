use egui::{CollapsingHeader, Context, ScrollArea, Ui, Window};
use petgraph::graph::NodeIndex;

use crate::simulation_controller::state::{DisplayOptions, State};

pub fn draw_infos_for_selected_nodes(ctx: &Context, state: &mut State) {
    let selected_nodes = state.graph_section.g.selected_nodes().to_owned();
    for node_index in selected_nodes {
        draw_node_info(ctx, node_index, state);
    }
}
pub fn draw_node_info(ctx: &Context, node_index: NodeIndex, state: &mut State) {
    Window::new(format!("Node {}", node_index.index()))
        .collapsible(true)
        .resizable(true)
        .default_width(200.0)
        .default_height(100.0)
        .show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                let payload = state.graph_section.g.node(node_index).unwrap().payload();
                let n = state.graph_section.g.node(node_index).unwrap();
                ui.label(
                    egui::RichText::new(format!("Type: {}", payload.get_type()))
                        .color(egui::Color32::LIGHT_GRAY),
                );
                ui.label(format!("Vendor: {}", n.payload().vendor));
                ui.horizontal(|ui| {
                    if ui.button("Close").clicked() {
                        let n = state
                            .graph_section
                            .g
                            .node_mut(node_index)
                            .unwrap()
                            .set_selected(false);
                    }
                    if ui.button("Hide others").clicked() {
                        for selected_node_index in state.graph_section.g.selected_nodes().to_owned()
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
                })
            });

            CollapsingHeader::new("Logs").show(ui, |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    for x in state
                        .events
                        .get_events_list(DisplayOptions::from_index(node_index))
                    {
                        ui.label(
                            egui::RichText::new(format!("{:?}", x))
                                .color(egui::Color32::LIGHT_GRAY),
                        );
                    }
                })
            })
        });
}

