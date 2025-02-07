use crate::simulation_controller::util::colors;
use egui::{Color32, Response, Ui};
use egui_extras::TableRow;
use wg_2024::{
    controller,
    packet::{self, NackType, PacketType},
};

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
            SCEventType::ClientEvent(client_event) => ui.label(""), /*TODO add more infos*/
            SCEventType::ServerEvent(server_event) => ui.label(""), /*TODO add more infos*/
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
                        PacketType::MsgFragment(fragment) => "MsgFragment",
                        PacketType::Ack(ack) => "Ack",
                        PacketType::Nack(nack) => "Nack",
                        PacketType::FloodRequest(flood_request) => "FloodRequest",
                        PacketType::FloodResponse(flood_response) => "FloodResponse",
                    }
                    .to_string();

                    let pack_type_string = match &packet.pack_type {
                        PacketType::MsgFragment(fragment) => {
                            ui.label(egui::RichText::new("MsgFragment").color(Color32::DARK_GRAY))
                        }
                        PacketType::Ack(ack) => {
                            ui.label(egui::RichText::new("Ack").color(Color32::DARK_GRAY))
                        }
                        PacketType::Nack(nack) => {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Nack").color(Color32::DARK_GRAY));
                                ui.label(egui::RichText::new(" - ").color(Color32::DARK_GRAY));
                                match nack.nack_type {
                                    NackType::ErrorInRouting(_) => ui.label(
                                        egui::RichText::new("ErrorInRouting")
                                            .color(Color32::DARK_GRAY),
                                    ),
                                    NackType::DestinationIsDrone => ui.label(
                                        egui::RichText::new("DestinationIsDrone")
                                            .color(Color32::DARK_GRAY),
                                    ),
                                    NackType::Dropped => ui.label(
                                        egui::RichText::new("Dropped").color(Color32::DARK_GRAY),
                                    ),
                                    NackType::UnexpectedRecipient(_) => ui.label(
                                        egui::RichText::new("UnexpectedRecipient")
                                            .color(Color32::DARK_GRAY),
                                    ),
                                };
                            })
                            .response
                        }
                        PacketType::FloodRequest(flood_request) => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("FloodRequest").color(Color32::DARK_GRAY),
                                );
                                ui.label(
                                    egui::RichText::new(format!(
                                        "Initiated by: {}, flood_id: {}",
                                        flood_request.initiator_id, flood_request.flood_id
                                    ))
                                    .color(Color32::DARK_GRAY),
                                );
                            })
                            .response
                        }
                        PacketType::FloodResponse(flood_response) => {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("FloodResponse").color(Color32::DARK_GRAY),
                                );
                                ui.label(
                                    egui::RichText::new(format!("{:?}", flood_response.path_trace))
                                        .color(Color32::DARK_GRAY),
                                )
                            })
                            .response
                        }
                    };

                    // ui.label(egui::RichText::new(pack_type_string).color(Color32::DARK_GRAY));
                })
                .response
            }
        }
    }
}
