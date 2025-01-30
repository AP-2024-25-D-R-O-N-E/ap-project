use std::{collections::HashMap, thread};

use colored::Colorize;
use crossbeam::channel::{select_biased, Receiver, Sender};
use petgraph::{
    prelude::{GraphMap, StableGraph},
    Undirected,
};
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType},
};

use crate::{
    fragmentation::{
        self,
        message::{self, Message},
        Fragmenter,
    },
    simulation_controller::structs::{ServerCommand, ServerEvent},
};

use super::ServerTrait;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum HashableNodeType {
    Drone,
    Server,
    Client,
}

#[derive(Debug)]
pub struct Server {
    pub id: NodeId,
    pub scs: Sender<ServerEvent>,
    pub scr: Receiver<ServerCommand>,
    pub pr: Receiver<Packet>,
    pub ps: HashMap<NodeId, Sender<Packet>>,
    floods_id: u64,
    topology: GraphMap<(NodeId, HashableNodeType), (), Undirected>,
    msg_buffer: Vec<Fragment>,
    session_id: u64,
    client_vector: Vec<NodeId>,
}

impl ServerTrait for Server {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ServerEvent>,
        sim_contr_recv: Receiver<ServerCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self {
        let mut topology: GraphMap<(NodeId, HashableNodeType), (), Undirected> = GraphMap::new();
        topology.add_node((id, HashableNodeType::Server));

        Server {
            id: id,
            scs: sim_contr_send,
            scr: sim_contr_recv,
            pr: packet_recv,
            ps: packet_send,
            floods_id: 0,
            topology,
            msg_buffer: Vec::new(),
            session_id: 0,
            client_vector: Vec::new(),
        }
    }

    fn run(&mut self) {
        loop {
            select_biased! {
                recv(self.scr) -> command_res => {
                    if let Ok(command) = command_res {
                        //here goes the handling fo the sim controller
                        match command {
                            ServerCommand::NetworkInitialized=>{
                                self.initiate_flood();log::debug!("{} at {} - network initialized"," <- network initialized".green(),self.id);}
                            ServerCommand::AddSender(node_id, sender) => self.add_sender(node_id, sender),
                            ServerCommand::RemoveSender(node_id) => self.remove_channel(node_id), 
                        }
                    }
                },
                recv(self.pr) -> packet_res => {
                    match packet_res {
                        //remember to remove the underscores when you actually start using the variable ig
                        Ok(packet) => {

                            log::debug!("{} at {} - packet: {:?}, {:?}", " <- packet received".green(), self.id, packet.session_id, packet.routing_header);

                            match packet.pack_type {
                                PacketType::Nack(nack)=>self.manage_nack(nack),
                                PacketType::Ack(ack)=>self.manage_ack(ack),
                                PacketType::MsgFragment(fragment)=>self.manage_msg_fragment(packet.session_id, fragment),
                                //  ...these two are jet to be defined...
                                PacketType::FloodRequest(_) => self.manage_flood_request(packet),
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
}

impl Fragmenter for Server {
    fn disassemble(msg: Message) -> std::collections::HashMap<u64, wg_2024::packet::Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> Message {
        todo!()
    }
}

impl Server {
    fn manage_nack(&self, nack: Nack) {
        //resend the packet
        log::debug!(
            "{} {} received a nack: {:?}",
            "↳ server".green(),
            self.id,
            nack
        );
    }

    fn manage_ack(&self, ack: Ack) {
        //free memory of message vector
        log::debug!(
            "{} {} received an ack: {:?}",
            "↳ server".green(),
            self.id,
            ack
        );
    }

    fn manage_msg_fragment(&mut self, session_id: u64, msg: Fragment) {
        //call to the assembler
        log::debug!(
            "{} {} received a fragment: {:?}",
            "↳ server".green(),
            self.id,
            msg
        );

        if self.session_id == 0 && self.msg_buffer.is_empty() {
            self.session_id = session_id;
            self.msg_buffer.reserve(msg.total_n_fragments as usize);
        }

        if session_id == self.session_id {
            let index = msg.fragment_index as usize;
            let total_frags = msg.total_n_fragments as usize;
            self.msg_buffer[index] = msg;

            if self.msg_buffer.len() == total_frags {
                let message = Self::assemble(self.msg_buffer.clone());
                //separate function in case I wanna multithread this later
                self.manage_message(message);
                self.msg_buffer.clear();
                self.session_id = 0;
            }
        } else {
            log::error!(
                "{} {} received a fragment with a different session id",
                "↳ server".red(),
                self.id
            );
        }
    }

    fn manage_message(&self, msg: Message) {}

    fn manage_flood_request(&self, mut packet: Packet) {
        log::debug!(
            "{} {} received a flood request: {:?}",
            "↳ server".green(),
            self.id,
            packet.pack_type
        );

        if let PacketType::FloodRequest(mut flood_request) = packet.pack_type {
            flood_request.path_trace.push((self.id, NodeType::Server));

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

    fn manage_flood_response(&mut self, fr: FloodResponse) {
        //call to the assembler
        log::debug!(
            "{} {} received a flood response: {:?}",
            "↳ server".green(),
            self.id,
            fr
        );

        // check that the flood response is ours
        if fr.path_trace[0].0 == self.id {
            let node = match fr.path_trace[0].1 {
                NodeType::Drone => (fr.path_trace[0].0, HashableNodeType::Drone),
                NodeType::Client => (fr.path_trace[0].0, HashableNodeType::Client),
                NodeType::Server => (fr.path_trace[0].0, HashableNodeType::Server),
            };

            let mut current_index = self.topology.add_node(node);

            for value in fr.path_trace.iter().skip(1) {
                let node = match value.1 {
                    NodeType::Drone => (value.0, HashableNodeType::Drone),
                    NodeType::Client => (value.0, HashableNodeType::Client),
                    NodeType::Server => (value.0, HashableNodeType::Server),
                };
                let new_node = self.topology.add_node(node);
                self.topology.add_edge(current_index, new_node, ());
                current_index = new_node;
            }
        }

        log::info!("{} {:?}", "Server topology: ".green(), self.topology);
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
            self.scs.send(ServerEvent::PacketSent(packet));
        }
    }

    fn initiate_flood(&mut self) {
        for (id, sender) in self.ps.iter() {
            let packet = Packet {
                pack_type: PacketType::FloodRequest(FloodRequest {
                    path_trace: vec![(self.id, NodeType::Server)],
                    flood_id: self.floods_id,
                    initiator_id: self.id,
                }),
                routing_header: SourceRoutingHeader {
                    hops: vec![],
                    hop_index: 0,
                },
                session_id: 0, // it'll be whatever for now
            };
            self.floods_id += 1;

            let res = sender.send(packet.clone());

            if let Err(mut packet) = res {
                log::error!("The send inside channel gave an error, this shouldn't be happening");
            } else {
                self.scs.send(ServerEvent::PacketSent(packet));
            }
        }
    }

    
    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.ps.insert(id, sender);
    }

    fn remove_channel(&mut self, id: NodeId) {
        self.ps.remove(&id);
    }
}
