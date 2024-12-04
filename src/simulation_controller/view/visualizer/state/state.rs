use super::{DebugSectionState, TestSectionState};

pub struct State {
    pub test_section: TestSectionState,
    pub debug_section: DebugSectionState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            test_section: Default::default(),
            debug_section: Default::default(),
        }
    }
}
