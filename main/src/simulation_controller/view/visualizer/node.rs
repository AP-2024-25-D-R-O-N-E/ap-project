use egui::{
    epaint::TextShape, text::Fonts, Color32, FontFamily, FontId, Pos2, Rect, Rounding, Shadow,
    Shape, Stroke, Vec2,
};
use egui_graphs::{DisplayNode, NodeProps};
use petgraph::{stable_graph::IndexType, EdgeType};

use crate::{initializer::drone_vendor::DroneVendor, simulation_controller::util};

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
pub struct UiNodePayload {
    pub node_type: UiNodeType,
    pub vendor: DroneVendor,
    pub wg_id: wg_2024::network::NodeId,
}
impl UiNodePayload {
    pub fn get_type(&self) -> String {
        match &self.node_type {
            UiNodeType::Server(ui_server_node) => "Server",
            UiNodeType::Client(ui_client_node) => "Client",
            UiNodeType::Drone(ui_drone_node) => "Drone",
        }
        .to_string()
    }
}

#[derive(Clone)]
pub enum UiNodeType {
    Server(UiServerNode),
    Client(UiClientNode),
    Drone(UiDroneNode),
}

#[derive(Clone)]
pub struct UiServerNode {}
#[derive(Clone)]
pub struct UiClientNode {}
#[derive(Clone)]
pub struct UiDroneNode {
    pub radius: f32,
    pub pdr: f32,
    pub last_committed_pdr: f32,
    pub crashed: bool,
}
impl UiDroneNode {
    pub fn new(pdr: f32) -> UiDroneNode {
        Self {
            radius: 23.,
            pdr,
            last_committed_pdr: pdr,
            crashed: false,
        }
    }
}
impl Default for UiDroneNode {
    fn default() -> Self {
        Self {
            radius: 20.,
            pdr: 0.,
            last_committed_pdr: 0.,
            crashed: false,
        }
    }
}

impl std::fmt::Display for UiNodePayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.node_type {
            UiNodeType::Server(_server_node) => write!(f, "Server[]"),
            UiNodeType::Client(_client_node) => write!(f, "Client[]"),
            UiNodeType::Drone(_drone_node) => write!(f, "Drone[]"),
        }
    }
}

#[derive(Clone)]
pub struct CustomNodeShape {
    pub label: String,
    pub loc: Pos2,
    pub selected: bool,
    pub payload: UiNodePayload,

    pub size_x: f32,
    pub size_y: f32,
    pub zoom: f32,
}

impl From<NodeProps<UiNodePayload>> for CustomNodeShape {
    fn from(node_props: NodeProps<UiNodePayload>) -> Self {
        // let id = node_props
        //     .label
        //     .chars()
        //     .filter(|c| c.is_digit(10)) // Keep only digits
        //     .collect::<String>() // Collect into a String
        //     .parse::<usize>()
        //     .ok();

        Self {
            label: node_props.label.clone(),
            loc: node_props.location(),
            selected: node_props.selected,
            payload: node_props.payload,

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
        let center = ctx.meta.canvas_to_screen_pos(self.loc);

        self.zoom = ctx.meta.zoom;
        let mut text = get_text(ctx, format!("Node {}", self.payload.wg_id), center);

        match &mut self.payload.node_type {
            UiNodeType::Server(_server_node) => {
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

                let rect_fill =
                    Shape::rect_filled(rect, rounding, util::colors::LIGHT_BACKGROUND_GRAY);
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
            UiNodeType::Client(_client_node) => {
                let rect = Rect {
                    min: Pos2::new(center.x - 20.0, center.y - 20.0),
                    max: Pos2::new(center.x + 20.0, center.y + 20.0),
                };
                let rect_fill = Shape::rect_filled(
                    rect,
                    Rounding::default(),
                    util::colors::LIGHT_BACKGROUND_GRAY,
                );
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
            UiNodeType::Drone(drone_node) => {
                let circle_fill = if drone_node.crashed {
                    Shape::circle_filled(center, drone_node.radius, Color32::from_rgb(130, 0, 0))
                } else {
                    Shape::circle_filled(
                        center,
                        drone_node.radius,
                        util::colors::DARK_BACKGROUND_GRAY,
                    )
                };

                let circle_stroke = if self.selected {
                    Shape::circle_stroke(center, drone_node.radius, Stroke::new(2., Color32::WHITE))
                } else {
                    Shape::circle_stroke(
                        center,
                        drone_node.radius,
                        Stroke::new(1., util::colors::LIGHT_BACKGROUND_GRAY),
                    )
                };

                let pdr_label =
                    get_text(ctx, drone_node.pdr.to_string(), center + Vec2::new(0., 5.));
                text.translate(Vec2::new(0., -5.));

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

                vec![
                    Shape::from(shadow),
                    circle_fill,
                    circle_stroke,
                    text,
                    pdr_label,
                ]
            }
        }

        // we need to offset label by half its size to place it in the center of the rect
    }
}

impl<E: Clone, Ty: EdgeType, Ix: IndexType> DisplayNode<UiNodePayload, E, Ty, Ix>
    for CustomNodeShape
{
    fn is_inside(&self, pos: Pos2) -> bool {
        match &self.payload.node_type {
            UiNodeType::Drone(drone_node) => {
                self.loc.distance(pos) <= drone_node.radius / self.zoom
            }
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

    fn update(&mut self, state: &NodeProps<UiNodePayload>) {
        self.label = state.label.clone();
        self.loc = state.location();
        self.selected = state.selected;
        self.payload = state.payload.clone();
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
