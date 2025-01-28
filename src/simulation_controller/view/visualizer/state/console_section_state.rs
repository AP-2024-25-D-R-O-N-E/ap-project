use std::collections::HashSet;

pub struct ConsoleSectionState {
    pub hovered_row: Option<usize>,
    pub selected_rows: HashSet<usize>,
}

impl Default for ConsoleSectionState {
    fn default() -> Self {
        Self {
            selected_rows: HashSet::new(),
            hovered_row: None,
        }
    }
}
