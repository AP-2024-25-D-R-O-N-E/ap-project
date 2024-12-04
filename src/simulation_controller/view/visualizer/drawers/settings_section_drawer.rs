use egui::{CollapsingHeader, Ui};

use crate::simulation_controller::state::State;

pub fn draw_section_settings(ui: &mut Ui, state: &mut State) {
    CollapsingHeader::new("Navigation")
        .default_open(true)
        .show(ui, |ui| {
            if ui
                .radio(
                    state
                        .settings_section
                        .navigation_settings
                        .fit_to_screen_enabled,
                    "Fit to screen",
                )
                .on_hover_text_at_pointer(
                    "Automatically fits the graph to screen\nDisables zooming",
                )
                .clicked()
            {
                state
                    .settings_section
                    .navigation_settings
                    .fit_to_screen_enabled = true;
                state
                    .settings_section
                    .navigation_settings
                    .zoom_and_pan_enabled = false;
            }

            if ui
                .radio(
                    state
                        .settings_section
                        .navigation_settings
                        .zoom_and_pan_enabled,
                    "Zoom + pan",
                )
                .on_hover_text_at_pointer("Move: left drag\nZoom: ctrl/cmd + scroll")
                .clicked()
            {
                state
                    .settings_section
                    .navigation_settings
                    .fit_to_screen_enabled = false;
                state
                    .settings_section
                    .navigation_settings
                    .zoom_and_pan_enabled = true;
            }
        });

    CollapsingHeader::new("Style").show(ui, |ui| {
        ui.checkbox(
            &mut state.settings_section.style_settings.labels_always,
            "labels_always",
        );
        ui.label("Wheter to show labels always or when interacted only.");
    });

    CollapsingHeader::new("Interaction").show(ui, |ui| {
                if ui.checkbox(&mut state.settings_section.interaction_settings.dragging_enabled, "dragging_enabled").clicked() && state.settings_section.interaction_settings.dragging_enabled {
                    state.settings_section.interaction_settings.node_clicking_enabled = true;
                };
                ui.label("To drag use LMB click + drag on a node.");

                ui.add_space(5.);

                ui.add_enabled_ui(!(state.settings_section.interaction_settings.dragging_enabled || state.settings_section.interaction_settings.node_selection_enabled || state.settings_section.interaction_settings.node_selection_multi_enabled), |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut state.settings_section.interaction_settings.node_clicking_enabled, "node_clicking_enabled");
                        ui.label("Check click events in last events");
                    }).response.on_disabled_hover_text("node click is enabled when any of the interaction is also enabled");
                });

                ui.add_space(5.);

                ui.add_enabled_ui(!state.settings_section.interaction_settings.node_selection_multi_enabled, |ui| {
                    ui.vertical(|ui| {
                        if ui.checkbox(&mut state.settings_section.interaction_settings.node_selection_enabled, "node_selection_enabled").clicked() && state.settings_section.interaction_settings.node_selection_enabled {
                            state.settings_section.interaction_settings.node_clicking_enabled = true;
                        };
                        ui.label("Enable select to select nodes with LMB click. If node is selected clicking on it again will deselect it.");
                    }).response.on_disabled_hover_text("node_selection_multi_enabled enables select");
                });

                if ui.checkbox(&mut state.settings_section.interaction_settings.node_selection_multi_enabled, "node_selection_multi_enabled").changed() && state.settings_section.interaction_settings.node_selection_multi_enabled {
                    state.settings_section.interaction_settings.node_clicking_enabled = true;
                    state.settings_section.interaction_settings.node_selection_enabled = true;
                }
                ui.label("Enable multiselect to select multiple nodes.");

                ui.add_space(5.);

                ui.add_enabled_ui(!(state.settings_section.interaction_settings.edge_selection_enabled || state.settings_section.interaction_settings.edge_selection_multi_enabled), |ui| {
                    ui.vertical(|ui| {
                        ui.checkbox(&mut state.settings_section.interaction_settings.edge_clicking_enabled, "edge_clicking_enabled");
                        ui.label("Check click events in last events");
                    }).response.on_disabled_hover_text("edge click is enabled when any of the interaction is also enabled");
                });

                ui.add_space(5.);

                ui.add_enabled_ui(!state.settings_section.interaction_settings.edge_selection_multi_enabled, |ui| {
                    ui.vertical(|ui| {
                        if ui.checkbox(&mut state.settings_section.interaction_settings.edge_selection_enabled, "edge_selection_enabled").clicked() && state.settings_section.interaction_settings.edge_selection_enabled {
                            state.settings_section.interaction_settings.edge_clicking_enabled = true;
                        };
                        ui.label("Enable select to select edges with LMB click. If edge is selected clicking on it again will deselect it.");
                    }).response.on_disabled_hover_text("edge_selection_multi_enabled enables select");
                });

                if ui.checkbox(&mut state.settings_section.interaction_settings.edge_selection_multi_enabled, "edge_selection_multi_enabled").changed() && state.settings_section.interaction_settings.edge_selection_multi_enabled {
                    state.settings_section.interaction_settings.edge_clicking_enabled = true;
                    state.settings_section.interaction_settings.edge_selection_enabled = true;
                }
                ui.label("Enable multiselect to select multiple edges.")
            });

    // CollapsingHeader::new("Selected")
    //     .default_open(true)
    //     .show(ui, |ui| {
    //         ScrollArea::vertical()
    //             .auto_shrink([false, true])
    //             .max_height(200.)
    //             .show(ui, |ui| {
    //                 state
    //                     .settings_section
    //                     .g
    //                     .selected_nodes()
    //                     .iter()
    //                     .for_each(|node| {
    //                         ui.label(format!("{node:?}"));
    //                     });
    //                 state
    //                     .settings_section
    //                     .g
    //                     .selected_edges()
    //                     .iter()
    //                     .for_each(|edge| {
    //                         ui.label(format!("{edge:?}"));
    //                     });
    //             });
    //     });
}
