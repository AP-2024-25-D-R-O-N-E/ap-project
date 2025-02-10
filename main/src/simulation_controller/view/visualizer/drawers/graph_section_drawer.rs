use egui::Ui;
use egui_graphs::{GraphView, LayoutRandom, LayoutStateRandom};

use crate::simulation_controller::{edge::UiEdgePayload, node::UiNodePayload, state::State};

pub fn draw_section_graph(ui: &mut Ui, state: &mut State) {
    let settings_interaction = &egui_graphs::SettingsInteraction::new()
        .with_node_selection_enabled(
            state
                .settings_section
                .interaction_settings
                .node_selection_enabled,
        )
        .with_node_selection_multi_enabled(
            state
                .settings_section
                .interaction_settings
                .node_selection_multi_enabled,
        )
        .with_dragging_enabled(state.settings_section.interaction_settings.dragging_enabled)
        .with_node_clicking_enabled(
            state
                .settings_section
                .interaction_settings
                .node_clicking_enabled,
        )
        .with_edge_clicking_enabled(
            state
                .settings_section
                .interaction_settings
                .edge_clicking_enabled,
        )
        .with_edge_selection_enabled(
            state
                .settings_section
                .interaction_settings
                .edge_selection_enabled,
        )
        .with_edge_selection_multi_enabled(
            state
                .settings_section
                .interaction_settings
                .edge_selection_multi_enabled,
        );
    let settings_navigation = &egui_graphs::SettingsNavigation::new()
        .with_zoom_and_pan_enabled(
            state
                .settings_section
                .navigation_settings
                .zoom_and_pan_enabled,
        )
        .with_fit_to_screen_enabled(
            state
                .settings_section
                .navigation_settings
                .fit_to_screen_enabled,
        )
        .with_zoom_speed(state.settings_section.navigation_settings.zoom_speed);
    let settings_style = &egui_graphs::SettingsStyle::new()
        .with_labels_always(state.settings_section.style_settings.labels_always);
    ui.add(
        &mut GraphView::<UiNodePayload, UiEdgePayload, _, _, _, _, LayoutStateRandom, LayoutRandom>::new(
            &mut state.graph_section.g,
        )
        .with_interactions(settings_interaction)
        .with_navigations(settings_navigation)
        .with_styles(settings_style)
        .with_events(&state.graph_section.graph_event_publisher),
    );
}
