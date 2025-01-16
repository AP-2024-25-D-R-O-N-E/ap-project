use egui::{
    epaint::TextShape, text::Fonts, Color32, FontFamily, FontId, Pos2, Rect, Rounding, Shadow,
    Shape, Stroke, Vec2,
};
use egui_graphs::{DisplayNode, NodeProps};
use petgraph::{stable_graph::IndexType, EdgeType};

fn get_text(ctx: &egui_graphs::DrawContext, text: String, pos: Pos2) -> Shape {
    ctx.ctx.fonts(|fonts| {
        Shape::text(
            fonts,
            pos,
            egui::Align2::CENTER_CENTER,
            text,
            // FontId::new(10., FontFamily::default()),
            FontId::proportional(10.),
            Color32::WHITE,
        )
    })
}
#[derive(Clone)]
pub struct NodePayload {
    pub node_type: NodeType,
}

#[derive(Clone)]
pub enum NodeType {
    Server(ServerNode),
    Client(ClientNode),
    Drone(DroneNode),
}

#[derive(Clone)]
pub struct ServerNode {}
#[derive(Clone)]
pub struct ClientNode {}
#[derive(Clone)]
pub struct DroneNode {
    radius: f32,
}
impl Default for DroneNode {
    fn default() -> Self {
        Self { radius: 15. }
    }
}

impl ToString for NodePayload {
    fn to_string(&self) -> String {
        match &self.node_type {
            NodeType::Server(_server_node) => format!("Server[]"),
            NodeType::Client(_client_node) => format!("Server[]"),
            NodeType::Drone(_drone_node) => format!("Server[]"),
        }
    }
}

#[derive(Clone)]
pub struct CustomNodeShape {
    pub node_type: NodeType,
    pub label: String,
    pub loc: Pos2,
    pub selected: bool,

    pub size_x: f32,
    pub size_y: f32,
    pub zoom: f32,
}

impl From<NodeProps<NodePayload>> for CustomNodeShape {
    fn from(node_props: NodeProps<NodePayload>) -> Self {
        Self {
            node_type: node_props.payload.node_type,
            label: node_props.label.clone(),
            loc: node_props.location.clone(),
            selected: node_props.selected,

            size_x: 20.,
            size_y: 20.,
            zoom: 1.,
        }
    }
}
trait DrawShape {
    fn draw_shape(&mut self, ctx: &egui_graphs::DrawContext) -> Vec<egui::Shape>;
}

impl DrawShape for CustomNodeShape {
    fn draw_shape(&mut self, ctx: &egui_graphs::DrawContext) -> Vec<egui::Shape> {
        // find node center location on the screen coordinates

        let center = ctx.meta.canvas_to_screen_pos(self.loc);
        // let text_color = ctx.ctx.style().visuals.text_color();
        // let backgound_color = ctx.ctx.style().visuals.extreme_bg_color;

        // create label
        // let label_bounding_box = ctx.ctx.fonts(|f| {
        //     f.layout_no_wrap(
        //         self.label.clone(),
        //         FontId::new(ctx.meta.canvas_to_screen_size(10.), FontFamily::default()),
        //         text_color,
        //     )
        // });
        //
        // let offset = Vec2::new(
        //     -label_bounding_box.size().x / 2.,
        //     -label_bounding_box.size().y / 2.,
        // );

        self.zoom = ctx.meta.zoom;
        // println!("{}", ctx.meta.zoom);
        match &mut self.node_type {
            NodeType::Server(_server_node) => {
                let text = get_text(ctx, self.label.clone(), center);
                let rect = Rect {
                    min: Pos2::new(center.x - 20.0, center.y - 20.0),
                    max: Pos2::new(center.x + 20.0, center.y + 20.0),
                };
                let rounding = Rounding {
                    nw: 10.,
                    ne: 10.,
                    sw: 10.,
                    se: 10.,
                };

                let rect_fill = Shape::rect_filled(rect, rounding, Color32::from_gray(45));
                let shadow = Shadow {
                    offset: Vec2::new(8., 8.),
                    blur: 30.,
                    spread: -10.,
                    color: Color32::from_gray(25),
                }
                .as_shape(rect, Rounding::ZERO);
                if self.selected {
                    let rect_stroke =
                        Shape::rect_stroke(rect, rounding, Stroke::new(2., Color32::WHITE));
                    vec![Shape::from(shadow), rect_fill, rect_stroke, text]
                } else {
                    vec![Shape::from(shadow), rect_fill, text]
                }
            }
            NodeType::Client(_client_node) => {
                let text = get_text(ctx, self.label.clone(), center);
                let rect = Rect {
                    min: Pos2::new(center.x - 20.0, center.y - 20.0),
                    max: Pos2::new(center.x + 20.0, center.y + 20.0),
                };
                let rect_fill =
                    Shape::rect_filled(rect, Rounding::default(), Color32::from_gray(45));
                let shadow = Shadow {
                    offset: Vec2::new(8., 8.),
                    blur: 30.,
                    spread: -10.,
                    color: Color32::BLACK,
                }
                .as_shape(rect, Rounding::ZERO);

                if self.selected {
                    let rect_stroke =
                        Shape::rect_stroke(rect, Rounding::ZERO, Stroke::new(2., Color32::WHITE));
                    vec![Shape::from(shadow), rect_fill, rect_stroke, text]
                } else {
                    vec![Shape::from(shadow), rect_fill, text]
                }
            }
            NodeType::Drone(drone_node) => {
                let text = get_text(ctx, self.label.clone(), center);
                let circle_fill =
                    Shape::circle_filled(center, drone_node.radius, Color32::from_gray(27));
                let circle_stroke = Shape::circle_stroke(
                    center,
                    drone_node.radius,
                    Stroke::new(1., Color32::DARK_GRAY),
                );

                let shadow = Shadow {
                    offset: Vec2::new(8., 8.),
                    blur: 30.,
                    spread: -10.,
                    color: Color32::BLACK,
                }
                .as_shape(
                    circle_fill.visual_bounding_rect(),
                    Rounding::same(circle_fill.visual_bounding_rect().width() / 2.),
                );

                vec![Shape::from(shadow), circle_fill, circle_stroke, text]
            }
        }

        // we need to offset label by half its size to place it in the center of the rect
    }
}

impl<E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<NodePayload, E, Ty, Ix>
    for CustomNodeShape
{
    fn is_inside(&self, pos: Pos2) -> bool {
        match &self.node_type {
            NodeType::Drone(drone_node) => self.loc.distance(pos) <= drone_node.radius / self.zoom,
            _ => {
                let rect = Rect::from_center_size(
                    self.loc,
                    Vec2::new(self.size_x / self.zoom, self.size_y / self.zoom),
                );
                rect.contains(pos)
            }
        }
    }

    fn closest_boundary_point(&self, _: Vec2) -> Pos2 {
        self.loc
    }

    fn shapes(&mut self, ctx: &egui_graphs::DrawContext) -> Vec<egui::Shape> {
        self.draw_shape(ctx)
    }

    fn update(&mut self, state: &NodeProps<NodePayload>) {
        self.label = state.label.clone();
        self.loc = state.location.clone();
        self.selected = state.selected;
    }
}

// fn find_intersection(center: Pos2, size_x: f32, size_y: f32, direction: Vec2) -> Pos2 {
//     if (direction.x.abs() * size_y) > (direction.y.abs() * size_x) {
//         // intersects left or right side
//         let x = if direction.x > 0.0 {
//             center.x + size_x / 2.0
//         } else {
//             center.x - size_x / 2.0
//         };
//         let y = center.y + direction.y / direction.x * (x - center.x);
//         Pos2::new(x, y)
//     } else {
//         // intersects top or bottom side
//         let y = if direction.y > 0.0 {
//             center.y + size_y / 2.0
//         } else {
//             center.y - size_y / 2.0
//         };
//         let x = center.x + direction.x / direction.y * (y - center.y);
//         Pos2::new(x, y)
//     }
// }
//
// fn rect_to_points(rect: Rect) -> Vec<Pos2> {
//     let top_left = rect.min;
//     let bottom_right = rect.max;
//     let top_right = Pos2::new(bottom_right.x, top_left.y);
//     let bottom_left = Pos2::new(top_left.x, bottom_right.y);
//
//     vec![top_left, top_right, bottom_right, bottom_left]
// }
