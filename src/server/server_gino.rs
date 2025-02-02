use std::{cell::RefCell, collections::HashMap, sync::{Arc, RwLock}, thread::{self, JoinHandle}};

use bincode::de::read;
use colored::Colorize;
use crossbeam::{channel::{select_biased, unbounded, Receiver, Sender}, select};
use petgraph::{
    prelude::{GraphMap, StableGraph},
    Undirected,
};
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{self, Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType},
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

pub struct ChatServer {
    id: NodeId,
    sim_contr_send: Sender<ServerEvent>,
    sim_contr_recv: Receiver<ServerCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: Arc<RwLock<HashMap<NodeId, Sender<Packet>>>>,
    flood_id: u64, // keeps track of the current flood index
    topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>, // nodes don't register type, as they're instead inside the client_table
    fragment_buffers: Arc<RwLock<HashMap<(NodeId, u64), Vec<Fragment>>>>, // stores fragments until they're ready to be assembled
}

impl ServerTrait for ChatServer {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ServerEvent>,
        sim_contr_recv: Receiver<ServerCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self
    where
        Self: Sized {
        todo!()
    }

    fn run(&mut self) {

        // thread handles for all threads spawned by this server
        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        // spawn the sender thread
        let packet_sender = self.packet_send.clone();
        let sim_contr_sender = self.sim_contr_send.clone();

        threads.push(thread::spawn(move || {
            ChatServer::sender_thread(packet_sender, sim_contr_sender);
        }));

        // spawn the message handling thread

        let fragment_buffers = self.fragment_buffers.clone();

        // ready channel to start the assembly of the fragments
        let (ready_send, ready_recv) = unbounded::<(NodeId, u64)>();

        threads.push(thread::spawn(move || {
            ChatServer::message_handler_thread(fragment_buffers, ready_recv);
        }));


        // at the end call the receiver thread (which is this one)
        self.receiver_thread(ready_send);

    }
}

impl Fragmenter for ChatServer {

    fn assemble(fragments: Vec<Fragment>) -> Message {
        todo!()
    }
    
    fn disassemble(msg: Message) -> HashMap<u64, Fragment> {
        todo!()
    }
}

impl ChatServer {

    fn receiver_thread(&mut self, ready_send: Sender<(NodeId, u64)>) {

        loop {
            select_biased!(
                recv(self.sim_contr_recv) -> cmd => {
                    if let Ok(command) = cmd {
                        match command {
                            ServerCommand::NetworkInitialized => todo!(),
                            ServerCommand::AddSender(_, sender) => todo!(),
                            ServerCommand::RemoveSender(_) => todo!(),
                        }
                    }
                },
                recv(self.packet_recv) -> res => {
                    if let Ok(mut packet) = res {
                        match packet.pack_type {
                            PacketType::MsgFragment(_) => self.manage_msg_fragment(packet, ready_send.clone()),
                            PacketType::Ack(ack) => todo!(),
                            PacketType::Nack(nack) => todo!(),
                            PacketType::FloodRequest(_) => self.manage_flood_request(packet),
                            PacketType::FloodResponse(flood_response) => self.manage_flood_response(flood_response),
                        }
                    }
                }
            )
        }
        


    }


    fn sender_thread(packet_sender: Arc<RwLock<HashMap<u8, Sender<Packet>>>>, sim_contr_send: Sender<ServerEvent>) {

    }

    fn message_handler_thread(fragment_buffers: Arc<RwLock<HashMap<(NodeId, u64), Vec<Fragment>>>>, ready_recv: Receiver<(NodeId, u64)>) {
        let mut client_table: HashMap<NodeId, NodeType> = HashMap::new();

        // basically infinite loop waiting for ready signal from the receiver thread
        while let Ok((source, msg_id)) = ready_recv.recv() {
            let mut fragment_buffers_lock = fragment_buffers.write().unwrap();

            // assemble the message
            if let Some(fragments) = fragment_buffers_lock.remove(&(source, msg_id)) {
                let mut message = Self::assemble(fragments);

                // check the message type and act accordingly
                let mut resp_message: Option<Message> = match message.message_data {
                    MessageData::RegisterAsClient(_) => todo!(),
                    MessageData::UnregisterAsClient(_) => todo!(),
                    MessageData::RequestClients(_) => todo!(),
                    MessageData::RequestHistory { requester, partner } => todo!(),
                    MessageData::TextMessage { from, to, text } => todo!(),
                    MessageData::FileMessage { from, to, file, file_name } => todo!(),
                    MessageData::ResponseClients(items) => todo!(),
                    MessageData::AcknolewdgedAsClient => todo!(),
                    MessageData::ResponseHistory { partner, history } => todo!(),
                    MessageData::UnregisteredSenderError => todo!(),
                    MessageData::UnregisteredRecipientError => todo!(),
                    MessageData::UnsupportedMessageTypeError => todo!(),
                };

                if let Some(resp_mess) = resp_message {
                    let fragments = Self::disassemble(resp_mess);

                    // send the fragments to the sender thread
                    todo!();

                }
            }
        }
    }

}

// Receiver thread functions    
impl ChatServer {

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
                    hop_index: 1,
                },
                session_id: 0, // it'll be whatever for now
            };

            self.send_packet(packet);
        }
    }
    
    fn manage_flood_response(&self, fr: FloodResponse) {
        log::debug!(
            "{} {} received a flood response: {:?}",
            "↳ server".green(),
            self.id,
            fr
        );

        // check that the flood response is ours
        if fr.path_trace[0].0 == self.id {

            let mut topology_lock = self.topology.write().unwrap();

            let mut current_index = topology_lock.add_node(self.id);

            for (node_id, node_type) in fr.path_trace.iter().skip(1) {
                let next_index = topology_lock.add_node(*node_id);

                topology_lock.add_edge(current_index, next_index, ());
                current_index = next_index;
            }
        }

        log::info!("{} {:?}", "Server topology: ".green(), self.topology);
    }

    fn manage_msg_fragment(&self, packet: Packet, ready_send: Sender<(NodeId, u64)>) {
        log::debug!(
            "{} {} received a message fragment: {:?}",
            "↳ server".green(),
            self.id,
            packet.pack_type
        );

        let packet_source = packet.routing_header.hops[0];
        let packet_msg_id = packet.session_id;

        // inverse route calculation for the ack
        let mut inverse_route = packet.routing_header.hops.clone();
        inverse_route.reverse();

        if let PacketType::MsgFragment(fragment) = packet.pack_type {

            // send ack for this fragment
            let packet = Packet {
                pack_type: PacketType::Ack(
                    Ack {
                        fragment_index: fragment.fragment_index,
                    }
                ),
                routing_header: SourceRoutingHeader {
                    hops: inverse_route,
                    hop_index: 1,
                },
                session_id: packet_msg_id,
            };

            self.send_packet(packet);

            // handle the fragment buffer and send the ready signal to the message handler thread
            let mut fragment_buffers_lock = self.fragment_buffers.write().unwrap();

            let total_frags = fragment.total_n_fragments;

            if fragment_buffers_lock.contains_key(&(packet_source, packet_msg_id)) {
                let mut frag_buffer = fragment_buffers_lock.get_mut(&(packet_source, packet_msg_id)).unwrap();
                
                frag_buffer.push(fragment);

                // if the buffer is full, tell the message handler thread to start assembling the fragments
                if frag_buffer.len() == total_frags as usize {
                    ready_send.send((packet_source, packet_msg_id));
                }

            } else {
                fragment_buffers_lock.insert((packet_source, packet_msg_id), vec![fragment]);
                // if the total frags is 1, then we can just send the message to the message handler thread
                if total_frags == 1 {
                    ready_send.send((packet_source, packet_msg_id));
                }
            }
        }
        
    }

    fn send_packet(&self, packet: Packet) {
        let next_node = packet.routing_header.hops[packet.routing_header.hop_index];
        let send_channel = &self.packet_send.read().unwrap()[&next_node];
    
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
            self.sim_contr_send.send(ServerEvent::PacketSent(packet));
        }
    }

}

// Sender thread functions

// Message handler thread functions
impl ChatServer {

}
