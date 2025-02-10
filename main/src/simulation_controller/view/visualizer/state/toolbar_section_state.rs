pub struct ToolbarSectionState {
    pub console_open: bool,
    pub test_open: bool,
    pub debug_open: bool,
    pub settings_open: bool,
    pub modify_topology_open: bool,
    pub handle_shortcuts: bool,
    pub events_json_open: bool,
    pub loggin_enabled: bool,
}

impl Default for ToolbarSectionState {
    fn default() -> Self {
        Self {
            console_open: true,
            test_open: true,
            debug_open: false,
            settings_open: false,
            modify_topology_open: false,
            handle_shortcuts: true,
            events_json_open: false,
            loggin_enabled: true,
        }
    }
}
