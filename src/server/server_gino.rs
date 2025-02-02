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
    fragment_buffers: Arc<RwLock<HashMap<u64, Vec<Fragment>>>>, // stores fragments until they're ready to be assembled
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
        let (ready_send, ready_recv) = unbounded::<NodeId>();

        threads.push(thread::spawn(move || {
            ChatServer::message_handler_thread(fragment_buffers, ready_recv);
        }));


        // at the end call the receiver thread (which is this one)
        self.receiver_thread(ready_send);

    }
}

impl Fragmenter for ChatServer {
    fn disassemble(msg: Message) -> HashMap<u64, Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<Fragment>) -> Message {
        todo!()
    }
}

impl ChatServer {

    fn receiver_thread(&mut self, ready_send: Sender<NodeId>) {

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

    fn message_handler_thread(fragment_buffers: Arc<RwLock<HashMap<u64, Vec<Fragment>>>>, ready_recv: Receiver<NodeId>) {
        let mut client_table: HashMap<NodeId, NodeType> = HashMap::new();



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

    fn manage_msg_fragment(&self, packet: Packet, ready_send: Sender<NodeId>) {
        log::debug!(
            "{} {} received a message fragment: {:?}",
            "↳ server".green(),
            self.id,
            packet.pack_type
        );

        if let PacketType::MsgFragment(fragment) = packet.pack_type {
            let fragment_buffers_lock = self.fragment_buffers.write().unwrap();

        }
        
    }

}

// Sender thread functions

// Message handler thread functions
impl ChatServer {

}
