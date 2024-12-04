use super::{DebugSectionState, GraphSectionState, SettingsSectionState, TestSectionState};

pub struct State {
    pub test_section: TestSectionState,
    pub debug_section: DebugSectionState,
    pub settings_section: SettingsSectionState,
    pub graph_section: GraphSectionState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            test_section: Default::default(),
            debug_section: Default::default(),
            settings_section: Default::default(),
            graph_section: Default::default(),
        }
    }
}
