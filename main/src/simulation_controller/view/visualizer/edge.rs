// use egui::epaint::*;
use egui::{epaint::PathStroke, Color32, Pos2, Shape, Stroke};
use egui_graphs::{DefaultEdgeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps, Node};
use petgraph::{stable_graph::IndexType, EdgeType};

use super::util::colors;

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

        let stroke_width = ctx.meta.canvas_to_screen_size(self.default_impl.width / 2.);

        if self.payload.is_active {
            vec![Shape::line_segment(
                points,
                Stroke::new(stroke_width, colors::OFF_WHITE),
            )]
        } else {
            Shape::dashed_line(
                &points,
                Stroke::new(stroke_width, colors::GRAY_WHITE),
                10.,
                5.,
            )
        }
    }

    fn update(&mut self, props: &egui_graphs::EdgeProps<UiEdgePayload>) {
        self.payload = props.payload.clone()
    }

    fn is_inside(
        &self,
        start: &Node<N, UiEdgePayload, Ty, Ix, D>,
        end: &Node<N, UiEdgePayload, Ty, Ix, D>,
        pos: Pos2,
    ) -> bool {
        self.default_impl.is_inside(start, end, pos)
    }
}
