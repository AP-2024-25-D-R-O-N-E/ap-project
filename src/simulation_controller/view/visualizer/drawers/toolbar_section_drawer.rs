use egui::{Layout, Response, Ui};

use crate::simulation_controller::{
    node::{UiDroneNode, UiNodePayload},
    state::State,
    util,
};

//Emojis 🧪📋🛠️❌📂
pub fn draw_toolbar_section(ui: &mut Ui, state: &mut State) {
    ui.horizontal(|ui| {
        // Toggle button: Test
        if ui
            .selectable_label(state.toolbar_section.test_open, "Test")
            .clicked()
        {
            state.toolbar_section.test_open = !state.toolbar_section.test_open;
        }

        ui.separator();

        // Toggle button: Console
        if ui
            .selectable_label(state.toolbar_section.console_open, "Console")
            .clicked()
        {
            state.toolbar_section.console_open = !state.toolbar_section.console_open;
        }

        ui.separator();

        // Toggle button: Settings
        if ui
            .selectable_label(state.toolbar_section.settings_open, "Settings")
            .clicked()
        {
            state.toolbar_section.settings_open = !state.toolbar_section.settings_open;
        }

        ui.separator();

        // Toggle button: Debug
        if ui
            .selectable_label(state.toolbar_section.debug_open, "Debug")
            .clicked()
        {
            state.toolbar_section.debug_open = !state.toolbar_section.debug_open;
        }

        ui.separator();
        // Regular button: Close all
        if ui.button("Close all").clicked() {
            state.node_info_section.opened_windows.clear();
        }

        // Regular button: Open selected
        if ui.button("Open selected").clicked() {
            let selected_nodes = state.graph_section.g.selected_nodes().to_owned();
            for node_index in selected_nodes {
                state
                    .node_info_section
                    .opened_windows
                    .insert(node_index, Default::default());
            }
        }

        // if ui.button("Spawn").clicked() {
        //     state.graph_section.g.add_node(UiNodePayload {
        //         node_type: UiDroneNode::default()
        //         vendor: todo!(),
        //         wg_id: todo!(),
        //     });
        // let selected_nodes = state.graph_section.g.selected_nodes().to_owned();
        // for node_index in selected_nodes {
        //     state.node_info_section.opened_windows.insert(node_index, Default::default() );
        // }
        // }

        match &state.graph_section.well_formedness_flag {
            Ok(_) => {
                add_right_aligned_label(
                    ui,
                    egui::RichText::new("Ok".to_string()).color(util::colors::MUTED_GREEN),
                )
                .on_hover_text("Topology is well-formed");
            }
            Err(err) => {
                add_right_aligned_label(
                    ui,
                    egui::RichText::new("Malformed topology".to_string())
                        .color(util::colors::MUTED_RED),
                )
                .on_hover_text(err);
            }
        }
    });
}

fn add_right_aligned_label(ui: &mut Ui, text: egui::RichText) -> Response {
    let label1 = egui::Label::new(text.clone());
    let label2 = egui::Label::new(text);
    let res = label1.layout_in_ui(ui).2;
    let label_width = res.intrinsic_size.unwrap().x;
    ui.add_space(ui.available_size().x - label_width);
    ui.add(label2)
}
