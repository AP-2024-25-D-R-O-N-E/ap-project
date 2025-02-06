use crate::simulation_controller::util::colors;
use egui::{Color32, Response, Ui};
use egui_extras::TableRow;
use wg_2024::{controller, packet};

use crate::simulation_controller::{
    serialization_ref_structs::IntoSerializable, state::State, SCEvent, SCEventType,
};

use super::{draw_all_events_as_json, draw_event_as_json};

impl SCEvent {
    pub fn draw(&self, row: &mut TableRow, state: &mut State) {
        row.col(|ui| {
            ui.label(format!(
                "{} {}",
                self.event_type.get_sender_type(),
                self.sender_id
            ));
        });

        row.col(|ui| {
            ui.label(self.event_type.get_event_type());
        });

        row.col(|ui| {
            ui.label(self.get_packet().session_id.to_string());
        });

        row.col(|ui| {
            // ui.label(self.event_type.get_packet_type());

            // let json_string = serde_json::to_string_pretty(&self).unwrap();
            // ui.label(format!("{:?}", self));
            // ui.horizontal(|ui| {
            // });
            //
            // if ui.button("?").clicked() {
            //     ui.to
            // }
            //     .on_hover_ui(|ui| {
            //     draw_event_as_json(ui, self);
            // });
            self.draw_additional_infos(ui);
        })
        .1
        .on_hover_ui(|ui| {
            draw_event_as_json(ui, self);
        });
    }

    fn draw_additional_infos(&self, ui: &mut Ui) -> Response {
        match &self.event_type {
            SCEventType::ClientEvent(client_event) => todo!(),
            SCEventType::ServerEvent(server_event) => todo!(),
            SCEventType::DroneEvent(drone_event) => {
                ui.horizontal(|ui| {
                    let (packet, response) = match drone_event {
                        controller::DroneEvent::PacketSent(packet) => (
                            packet,
                            ui.label(egui::RichText::new("Packet sent").color(colors::MUTED_GREEN)),
                        ),
                        controller::DroneEvent::PacketDropped(packet) => (
                            packet,
                            ui.label(egui::RichText::new("Packet dropped").color(colors::YELLOW)),
                        ),
                        controller::DroneEvent::ControllerShortcut(packet) => (
                            packet,
                            ui.label(
                                egui::RichText::new("Controller shortcut")
                                    .color(colors::METALLIC_BLUE),
                            ),
                        ),
                    };

                    let pack_type_string = match &packet.pack_type {
                        packet::PacketType::MsgFragment(fragment) => "MsgFragment",
                        packet::PacketType::Ack(ack) => "Ack",
                        packet::PacketType::Nack(nack) => "Nack",
                        packet::PacketType::FloodRequest(flood_request) => "FloodRequest",
                        packet::PacketType::FloodResponse(flood_response) => "FloodResponse",
                    }
                    .to_string();

                    ui.label(egui::RichText::new(pack_type_string).color(Color32::DARK_GRAY));
                })
                .response
            }
        }
    }
}
