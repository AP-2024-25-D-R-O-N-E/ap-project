use std::{collections::HashMap, thread};

use colored::Colorize;
use crossbeam::{
    channel::{select_biased, unbounded, Receiver, Sender},
    select,
};
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
        message::{self, ChatMessage, Message, MessageData},
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
    msg_buffer: HashMap<NodeId, Vec<Fragment>>,
    client_vector: Vec<NodeId>,
    clients_history: HashMap<(NodeId, NodeId), Box<Vec<ChatMessage>>>,
    session_id: u64,
    threads: Vec<thread::JoinHandle<()>>,
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
            msg_buffer: HashMap::new(),
            session_id: 0,
            client_vector: Vec::new(),
            clients_history: HashMap::new(),
            threads: Vec::new(),
        }
    }

    fn run(&mut self) {
        // reading thread
        // sending messages thread
        // handle messages logic thread

        // channel used to send fragments of messages to get handled
        let mut message_handling_channel = unbounded::<(NodeId, u64, Fragment)>();

        // channel used to be able to send messages at the same time as reading them, otherwise you wouldn't be able to change a route if you get an ack before
        // you finish sending the fragments
        let mut packet_send_channel = unbounded::<PacketType>();

        // message handling thread
        self.threads.push(thread::spawn(move || loop {
            select! {
                recv(message_handling_channel.1) -> fragment_res => {
                    if let Ok((origin_id, session_id, fragment)) = fragment_res {




                    }
                },
            }
        }));

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
                                PacketType::MsgFragment(fragment)=>self.manage_msg_fragment(packet.routing_header.hops[0], packet.session_id, fragment),
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
    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> Message {
        todo!()
    }

    fn disassemble(msg: Message) -> std::collections::VecDeque<Fragment> {
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

    fn manage_msg_fragment(&mut self, origin_id: NodeId, session_id: u64, msg: Fragment) {
        //call to the assembler
        log::debug!(
            "{} {} received a fragment: {:?}",
            "↳ server".green(),
            self.id,
            msg
        );

        if self.client_vector.contains(&origin_id) {
            let message = Message::new(self.id, origin_id, MessageData::UnregisteredSenderError);

            // send message

            return;
        }

        let client_buffer = self.msg_buffer.get_mut(&origin_id).unwrap();

        if self.session_id == 0 && client_buffer.is_empty() {
            self.session_id = session_id;
            client_buffer.reserve(msg.total_n_fragments as usize);
        }

        if session_id == self.session_id {
            let index = msg.fragment_index as usize;
            let total_frags = msg.total_n_fragments as usize;
            client_buffer[index] = msg;

            if client_buffer.len() == total_frags {
                let message = Self::assemble(client_buffer.clone());
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

    fn manage_message(&mut self, msg: Message) {
        match msg.message_data {
            message::MessageData::RegisterAsClient(id) => {
                self.client_vector.push(id);
                todo!();
            }
            message::MessageData::UnregisterAsClient(id) => {
                self.client_vector.retain(|&x| x != id);
            }
            message::MessageData::RequestClients(id) => {
                todo!()
            }
            message::MessageData::RequestHistory { requester, partner } => todo!(),
            message::MessageData::TextMessage { from, to, text } => todo!(),
            message::MessageData::FileMessage {
                from,
                to,
                file,
                file_name,
            } => todo!(),
            message::MessageData::ResponseClients(items) => todo!(),
            message::MessageData::AcknolewdgedAsClient => todo!(),
            message::MessageData::ResponseHistory { partner, history } => todo!(),
            message::MessageData::UnregisteredSenderError => todo!(),
            message::MessageData::UnregisteredRecipientError => todo!(),
            message::MessageData::UnsupportedMessageTypeError => todo!(),
        }
    }

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

    fn create_message(&self, destination_id: NodeId, message_data: MessageData) -> Message {
        Message::new(self.id, destination_id, message_data)
    }

    fn find_route(&self, destination_id: NodeId) -> SourceRoutingHeader {
        // do stuff

        SourceRoutingHeader {
            hops: vec![],
            hop_index: 0,
        }
    }

    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.ps.insert(id, sender);
    }

    fn remove_channel(&mut self, id: NodeId) {
        self.ps.remove(&id);
    }
}
