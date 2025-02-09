pub struct DebugSectionState {
    pub zoom: f32,
    pub pan: [f32; 2],
    pub fps: f32,
    pub graph_events: Vec<String>,
}

impl Default for DebugSectionState {
    fn default() -> Self {
        Self {
            zoom: 0.,
            pan: [0., 0.],
            fps: 100.,
            graph_events: vec![],
        }
    }
}
