use crate::simulation_controller::{util::colors, ClientEvent, ServerEvent};
use egui::{Color32, Response, Ui};
use egui_extras::TableRow;
use wg_2024::{
    controller,
    packet::{NackType, Packet, PacketType},
};

use crate::simulation_controller::{SCEvent, SCEventType};

use super::draw_event_as_json;

impl SCEvent {
    pub fn draw(&self, row: &mut TableRow) {
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
            match self.get_packet() {
                Some(packet) => ui.label(packet.session_id.to_string()),
                None => ui.label("None"),
            };
        });

        row.col(|ui| {
            self.draw_additional_infos(ui);
        })
        .1
        .on_hover_ui(|ui| {
            draw_event_as_json(ui, self);
        });
    }

    fn draw_additional_infos(&self, ui: &mut Ui) -> Response {
        match &self.event_type {
            SCEventType::Client(client_event) => match client_event {
                ClientEvent::PacketSent(_) => {
                    ui.label(egui::RichText::new("Packet sent").color(colors::MUTED_GREEN))
                }
                ClientEvent::PacketReceived(_) => {
                    ui.label(egui::RichText::new("Packet received").color(colors::MUTED_GREEN))
                }
                ClientEvent::TextMessage { from, text, .. } => ui.label(
                    egui::RichText::new(format!("Recieved text message from {from}\n{text}"))
                        .color(colors::METALLIC_BLUE),
                ),
                ClientEvent::FileMessage {
                    from, file_path, ..
                } => ui.label(
                    egui::RichText::new(format!(
                        "Recieved file message from {from}\nFile path: {file_path:?}"
                    ))
                    .color(colors::METALLIC_BLUE),
                ),
                ClientEvent::ResponseClientsReceived(vec) => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new("Fetched available clients:".to_string())
                                .color(colors::MUTED_GREEN),
                        );
                        ui.label(egui::RichText::new(format!("{vec:?}")));
                    })
                    .response
                }
                ClientEvent::AcknolewdgedAsClient => {
                    ui.label(egui::RichText::new("AcknolewdgedAsClient").color(colors::MUTED_GREEN))
                }
                ClientEvent::ResponseHistoryReceived { partner, history } => {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("Fetched history from: {partner}"))
                                .color(colors::MUTED_GREEN),
                        );
                        ui.label(egui::RichText::new(format!("{history:?}")));
                    })
                    .response
                }
                ClientEvent::UnregisteredSenderError => ui
                    .label(egui::RichText::new("UnregisteredSenderError").color(colors::MUTED_RED)),
                ClientEvent::UnregisteredRecipientError => ui.label(
                    egui::RichText::new("UnregisteredRecipientError").color(colors::MUTED_RED),
                ),
                ClientEvent::UnsupportedMessageTypeError => ui.label(
                    egui::RichText::new("UnsupportedMessageTypeError").color(colors::MUTED_RED),
                ),
                ClientEvent::CreatedFileLocal {
                    from, file_path, ..
                } => ui.label(
                    egui::RichText::new(format!(
                        "Client {from} created local file\nFile path: {file_path:?}"
                    ))
                    .color(colors::METALLIC_BLUE),
                ),
            },

            SCEventType::Server(server_event) => match server_event {
                ServerEvent::PacketSent(packet) => {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("PacketSent").color(colors::MUTED_GREEN));
                        display_packet_infos(ui, packet);
                    })
                    .response
                }
                ServerEvent::PacketReceived(packet) => {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("PacketReceived").color(colors::MUTED_GREEN));
                        display_packet_infos(ui, packet);
                    })
                    .response
                }
            },
            SCEventType::Drone(drone_event) => {
                ui.horizontal(|ui| {
                    let (packet, _) = match drone_event {
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
                    display_packet_infos(ui, packet)
                    // ui.label(egui::RichText::new(pack_type_string).color(Color32::DARK_GRAY));
                })
                .response
            }
        }
    }
}

fn display_packet_infos(ui: &mut Ui, packet: &Packet) -> Response {
    match &packet.pack_type {
        PacketType::MsgFragment(_) => {
            ui.label(egui::RichText::new("MsgFragment").color(Color32::DARK_GRAY))
        }
        PacketType::Ack(_) => ui.label(egui::RichText::new("Ack").color(Color32::DARK_GRAY)),
        PacketType::Nack(nack) => {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Nack").color(Color32::DARK_GRAY));
                ui.label(egui::RichText::new(" - ").color(Color32::DARK_GRAY));
                match nack.nack_type {
                    NackType::ErrorInRouting(_) => {
                        ui.label(egui::RichText::new("ErrorInRouting").color(Color32::DARK_GRAY))
                    }
                    NackType::DestinationIsDrone => ui
                        .label(egui::RichText::new("DestinationIsDrone").color(Color32::DARK_GRAY)),
                    NackType::Dropped => {
                        ui.label(egui::RichText::new("Dropped").color(Color32::DARK_GRAY))
                    }
                    NackType::UnexpectedRecipient(_) => ui.label(
                        egui::RichText::new("UnexpectedRecipient").color(Color32::DARK_GRAY),
                    ),
                };
            })
            .response
        }
        PacketType::FloodRequest(flood_request) => {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("FloodRequest").color(Color32::DARK_GRAY));
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
                ui.label(egui::RichText::new("FloodResponse").color(Color32::DARK_GRAY));
                ui.label(
                    egui::RichText::new(format!("{:?}", flood_response.path_trace))
                        .color(Color32::DARK_GRAY),
                )
            })
            .response
        }
    }
}
