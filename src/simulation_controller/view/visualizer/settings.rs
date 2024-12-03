pub struct SettingsGraph {
    pub count_node: usize,
    pub count_edge: usize,
}

impl Default for SettingsGraph {
    fn default() -> Self {
        Self {
            count_node: 25,
            count_edge: 50,
        }
    }
}

pub struct SettingsInteraction {
    pub dragging_enabled: bool,
    pub node_clicking_enabled: bool,
    pub node_selection_enabled: bool,
    pub node_selection_multi_enabled: bool,
    pub edge_clicking_enabled: bool,
    pub edge_selection_enabled: bool,
    pub edge_selection_multi_enabled: bool,
}

impl Default for SettingsInteraction {
    fn default() -> Self {
        SettingsInteraction {
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

pub struct SettingsNavigation {
    pub fit_to_screen_enabled: bool,
    pub zoom_and_pan_enabled: bool,
    pub zoom_speed: f32,
}

impl Default for SettingsNavigation {
    fn default() -> Self {
        Self {
            zoom_speed: 0.05,
            fit_to_screen_enabled: false,
            zoom_and_pan_enabled: true,
        }
    }
}

pub struct SettingsStyle {
    pub labels_always: bool,
}

impl Default for SettingsStyle {
    fn default() -> Self {
        SettingsStyle {
            labels_always: true,
        }
    }
}

pub struct SettingsSimulation {
    pub dt: f32,
    pub cooloff_factor: f32,
    pub scale: f32,
}

impl Default for SettingsSimulation {
    fn default() -> Self {
        Self {
            dt: 0.03,
            cooloff_factor: 0.85,
            scale: 100.,
        }
    }
}
