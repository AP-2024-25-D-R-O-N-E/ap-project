use egui::{CollapsingHeader, Context, ScrollArea, Ui, Window};
use petgraph::graph::NodeIndex;

use crate::simulation_controller::state::{DisplayOptions, State};

pub fn draw_infos_for_selected_nodes(ctx: &Context, state: &mut State) {
    let selected_nodes = state.graph_section.g.selected_nodes().to_owned();
    // println!("{:?}", selected_nodes);
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
                ui.label(node_index.index().to_string());
                if ui.button("Close").clicked() {
                    // let new_selected_nodes = state
                    //     .graph_section
                    //     .g
                    //     .selected_nodes()
                    //     .to_owned()
                    //     .into_iter()
                    //     .filter(|n| *n != node_index)
                    //     .collect();
                    // println!("{:?}", new_selected_nodes);
                    // state.graph_section.g.set_selected_nodes(new_selected_nodes);
                    let n = state
                        .graph_section
                        .g
                        .node_mut(node_index)
                        .unwrap()
                        .set_selected(false);
                }
            });

            let events = state.events.get_events_list(DisplayOptions::ALL);
            // println!("{:?}", events.len());
            CollapsingHeader::new("Logs").show(ui, |ui| {
                for x in state.events.get_events_list(DisplayOptions::ALL) {
                    ui.label(
                        egui::RichText::new(format!("{:?}", x)).color(egui::Color32::LIGHT_GRAY),
                    );
                }
            })
        });
}
