
use std::collections::HashMap;

use colored::Colorize;
use crossbeam::channel::{select_biased, Receiver, Sender};
use wg_2024::{
    network::NodeId,
    packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, Packet, PacketType},
};

use crate::{
    fragmentation::{message::Message, Fragmenter},
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

use super::ClientTrait;

#[derive(Debug)]
pub struct Client {
    pub id: NodeId,
    pub scs: Sender<ClientEvent>,
    pub scr: Receiver<ClientCommand>,
    pub pr: Receiver<Packet>,
    pub ps: HashMap<NodeId, Sender<Packet>>,
}

impl ClientTrait for Client {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ClientEvent>,
        sim_contr_recv: Receiver<ClientCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        Client {
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
                                PacketType::FloodRequest(flood_request) => self.manage_flood_request(flood_request),
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

impl Fragmenter for Client {
    fn disassemble(
        msg: Message,
    ) -> std::collections::HashMap<u64, wg_2024::packet::Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> Message {
        todo!()
    }
}

impl Client {

    fn manage_nack(&self, nack: &Nack) {
        //resend the packet
        log::debug!("{} {} received a nack: {:?}", "↳ client".green(), self.id, nack);
    }

    fn manage_ack(&self, ack: &Ack) {
        //free memory of message vector
        log::debug!("{} {} received an ack: {:?}", "↳ client".green(), self.id, ack);
    }

    fn manage_msg_fragment(&self, msg: &Fragment) {
        //call to the assembler
        log::debug!("{} {} received a fragment: {:?}", "↳ client".green(), self.id, msg);
    }

    fn manage_flood_request(&self, fr: &FloodRequest) {
        //call to the assembler
        log::debug!("{} {} received a flood request: {:?}", "↳ client".green(), self.id, fr);
    }

    fn manage_flood_response(&self, fr: &FloodResponse) {
        //call to the assembler
        log::debug!("{} {} received a flood response: {:?}", "↳ client".green(), self.id, fr);
    }
}
