#[derive(Default)]
pub struct SettingsSectionState {
    pub interaction_settings: InteractionSettings,
    pub navigation_settings: NavigationSettings,
    pub style_settings: StyleSettings,
}

pub struct InteractionSettings {
    pub dragging_enabled: bool,
    pub node_clicking_enabled: bool,
    pub node_selection_enabled: bool,
    pub node_selection_multi_enabled: bool,
    pub edge_clicking_enabled: bool,
    pub edge_selection_enabled: bool,
    pub edge_selection_multi_enabled: bool,
}

impl Default for InteractionSettings {
    fn default() -> Self {
        InteractionSettings {
            dragging_enabled: true,
            node_clicking_enabled: true,
            node_selection_enabled: true,
            node_selection_multi_enabled: true,
            edge_clicking_enabled: false,
            edge_selection_enabled: false,
            edge_selection_multi_enabled: false,
        }
    }
}

pub struct NavigationSettings {
    pub fit_to_screen_enabled: bool,
    pub zoom_and_pan_enabled: bool,
    pub zoom_speed: f32,
}

impl Default for NavigationSettings {
    fn default() -> Self {
        Self {
            zoom_speed: 0.05,
            fit_to_screen_enabled: false,
            zoom_and_pan_enabled: true,
        }
    }
}

pub struct StyleSettings {
    pub labels_always: bool,
}

impl Default for StyleSettings {
    fn default() -> Self {
        StyleSettings {
            labels_always: true,
        }
    }
}
