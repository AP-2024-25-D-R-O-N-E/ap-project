use std::collections::HashSet;

#[derive(Default)]
pub struct ConsoleSectionState {
    pub hovered_row: Option<usize>,
    pub selected_rows: HashSet<usize>,
}

