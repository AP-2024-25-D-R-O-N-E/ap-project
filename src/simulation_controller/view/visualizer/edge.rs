// use egui::epaint::*;
use egui::{Color32, Pos2, Shape};
use egui_graphs::{DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node};
use petgraph::{stable_graph::IndexType, EdgeType};

#[derive(Clone)]
pub struct UiEdgePayload {
    pub is_active: bool,
}
impl Default for UiEdgePayload {
    fn default() -> Self {
        Self { is_active: true }
    }
}

#[derive(Clone)]
pub struct CustomEdgeShape {
    default_impl: DefaultEdgeShape,
    payload: UiEdgePayload,
}

impl From<EdgeProps<UiEdgePayload>> for CustomEdgeShape {
    fn from(props: EdgeProps<UiEdgePayload>) -> Self {
        Self {
            payload: props.payload.clone(),
            default_impl: DefaultEdgeShape::from(props),
        }
    }
}

impl<N: Clone, Ty: EdgeType, Ix: IndexType, D: DisplayNode<N, UiEdgePayload, Ty, Ix>>
    DisplayEdge<N, UiEdgePayload, Ty, Ix, D> for CustomEdgeShape
{
    fn shapes(
        &mut self,
        start: &Node<N, UiEdgePayload, Ty, Ix, D>,
        end: &Node<N, UiEdgePayload, Ty, Ix, D>,
        ctx: &DrawContext,
    ) -> Vec<egui::Shape> {
        let (start, end) = (start.location(), end.location());
        let points = [start, end].map(|p| ctx.meta.canvas_to_screen_pos(p));
        let dotted_line = Shape::dotted_line(&points, Color32::WHITE, 10., 2.);
        dotted_line
    }

    fn update(&mut self, _: &egui_graphs::EdgeProps<UiEdgePayload>) {}

    fn is_inside(
        &self,
        start: &Node<N, UiEdgePayload, Ty, Ix, D>,
        end: &Node<N, UiEdgePayload, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        self.default_impl.is_inside(start, end, pos)
    }
}
