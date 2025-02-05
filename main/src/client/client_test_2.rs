// this client is only a test to showcase having multiple client definitions

use std::collections::HashMap;

use colored::Colorize;
use crossbeam::channel::{select_biased, Receiver, Sender};
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType},
};

use crate::{
    fragmentation::{message::Message, Fragmenter},
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

use super::ClientTrait;

#[derive(Debug)]
pub struct Client2 {
    pub id: NodeId,
    pub scs: Sender<ClientEvent>,
    pub scr: Receiver<ClientCommand>,
    pub pr: Receiver<Packet>,
    pub ps: HashMap<NodeId, Sender<Packet>>,
}

impl ClientTrait for Client2 {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ClientEvent>,
        sim_contr_recv: Receiver<ClientCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        Client2 {
            id: id,
            scs: sim_contr_send,
            scr: sim_contr_recv,
            pr: packet_recv,
            ps: packet_send,
        }
    }

    fn run(&mut self) {
        loop {
            select_biased! {
                recv(self.scr) -> command_res => {
                    if let Ok(command) = command_res {
                        //here goes the handling fo the sim controller
                    }
                },
                recv(self.pr) -> packet_res => {

                    match packet_res {
                        //remember to remove the underscores when you actually start using the variable ig
                        Ok(packet) => {
                            log::debug!("{} at {} - packet: {:?}, {:?}", " <- packet received".green(), self.id, packet.session_id, packet.routing_header);

                            match &packet.pack_type {
                                PacketType::Nack(nack)=>self.manage_nack(nack),
                                PacketType::Ack(ack)=>self.manage_ack(ack),
                                PacketType::MsgFragment(fragment)=>self.manage_msg_fragment(fragment),
                                //  ...these two are jet to be defined...
                                PacketType::FloodRequest(flood_request) => self.manage_flood_request(packet),
                                PacketType::FloodResponse(flood_response) => self.manage_flood_response(flood_response),
                            }
                        },
                        Err(error) => {
                            log::info!("Necessary error at program end: {}", error);
                            return;
                        },
                    }

                },

            }
        }
    }

    // these are all sample function to handle messages
}

impl Fragmenter for Client2 {
    fn disassemble(msg: Message) -> std::collections::HashMap<u64, wg_2024::packet::Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> Message {
        todo!()
    }
}

impl Client2 {
    fn manage_nack(&self, nack: &Nack) {
        //resend the packet
        log::debug!(
            "{} {} received a nack: {:?}",
            "↳ client".green(),
            self.id,
            nack
        );
    }

    fn manage_ack(&self, ack: &Ack) {
        //free memory of message vector
        log::debug!(
            "{} {} received an ack: {:?}",
            "↳ client".green(),
            self.id,
            ack
        );
    }

    fn manage_msg_fragment(&self, msg: &Fragment) {
        //call to the assembler
        log::debug!(
            "{} {} received a fragment: {:?}",
            "↳ client".green(),
            self.id,
            msg
        );
    }

    fn manage_flood_request(&self, mut packet: Packet) {
        log::debug!(
            "{} {} received a flood request: {:?}",
            "↳ client".green(),
            self.id,
            packet.pack_type
        );

        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type {
            flood_request.path_trace.push((self.id, NodeType::Client));

            let new_flood_res = FloodResponse {
                path_trace: flood_request.path_trace,
                flood_id: flood_request.flood_id,
            };

            // creates inverted route starting from path_trace
            let mut inverse_route: Vec<NodeId> =
                new_flood_res.path_trace.iter().map(|(id, _)| *id).collect();
            // ignore the rare occurrances where a loop might be created as its not computationally viable to consider it
            inverse_route.reverse();

            let packet = Packet {
                pack_type: PacketType::FloodResponse(new_flood_res),
                routing_header: SourceRoutingHeader {
                    hops: inverse_route,
                    hop_index: 0,
                },
                session_id: 0, // it'll be whatever for now
            };

            self.forward_packet(packet);
        }
    }

    fn manage_flood_response(&self, fr: &FloodResponse) {
        //call to the assembler
        log::debug!(
            "{} {} received a flood response: {:?}",
            "↳ client".green(),
            self.id,
            fr
        );
    }

    fn forward_packet(&self, mut packet: Packet) {
        packet.routing_header.hop_index += 1;
        let next_node = packet.routing_header.hops[packet.routing_header.hop_index];

        // finds the channel corresponding to the next node without any checks, since they were done previously
        let send_channel = &self.ps[&next_node];

        log::debug!(
            "{} from {} - packet: {}",
            " -> packet sent ".blue(),
            self.id,
            packet
        );

        let res = send_channel.send(packet.clone());

        if let Err(mut packet) = res {
            log::error!("The send inside channel gave an error, this shouldn't be happening");
        } else {
            self.scs.send(ClientEvent::PacketSent(packet));
        }
    }
}
