pub struct ToolbarSectionState {
    pub console_open: bool,
    pub test_open: bool,
    pub debug_open: bool,
    pub settings_open: bool,
}

impl Default for ToolbarSectionState {
    fn default() -> Self {
        Self {
            console_open: false,
            test_open: false,
            debug_open: false,
            settings_open: false,
        }
    }
}
