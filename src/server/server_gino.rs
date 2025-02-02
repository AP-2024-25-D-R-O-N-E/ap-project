use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    sync::{Arc, Condvar, Mutex, RwLock},
    thread::{self, JoinHandle},
};

use bincode::de::read;
use colored::Colorize;
use crossbeam::{
    channel::{select_biased, unbounded, Receiver, Sender},
    select,
};
use petgraph::{
    algo,
    prelude::{GraphMap, StableGraph},
    Undirected,
};
use serde::de::DeserializeSeed;
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{
        self, Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType,
    },
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
    ack_packet_buffer: Arc<Mutex<HashMap<(NodeId, u64, u64), Packet>>>, // stores packets that need to await an ack. The tuple is (destination, session_id, frag_index)
    topology_modified: Arc<Mutex<bool>>, // flag to check if the topology has been modified
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
        Self: Sized,
    {
        todo!()
    }

    fn run(&mut self) {
        // thread handles for all threads spawned by this server
        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        // spawn the sender thread
        let packet_sender = self.packet_send.clone();
        let sim_contr_sender = self.sim_contr_send.clone();

        let (nack_send, nack_recv) = unbounded::<(NodeId, u64, Fragment)>();

        let (fragment_send, fragment_recv) = unbounded::<(NodeId, u64, Fragment)>();

        let condv = Condvar::new();

        let ack_packet_buffer = self.ack_packet_buffer.clone();

        let topology = self.topology.clone();

        let id = self.id;

        let topology_modified = self.topology_modified.clone();

        threads.push(thread::spawn(move || {
            ChatServer::sender_thread(
                id,
                packet_sender,
                sim_contr_sender,
                fragment_recv,
                nack_recv,
                condv,
                ack_packet_buffer,
                topology,
                topology_modified,
            );
        }));

        // spawn the message handling thread
        let fragment_buffers = self.fragment_buffers.clone();

        // ready channel to start the assembly of the fragments
        let (ready_send, ready_recv) = unbounded::<(NodeId, u64)>();

        threads.push(thread::spawn(move || {
            ChatServer::message_handler_thread(fragment_buffers, ready_recv, fragment_send);
        }));

        // at the end call the receiver thread (which is this one)
        self.receiver_thread(ready_send, nack_send);
    }
}

impl Fragmenter for ChatServer {
    fn assemble(fragments: Vec<Fragment>) -> Message {
        todo!()
    }

    fn disassemble(msg: Message) -> std::collections::VecDeque<Fragment> {
        todo!()
    }
}

impl ChatServer {
    fn receiver_thread(
        &mut self,
        ready_send: Sender<(NodeId, u64)>,
        nack_send: Sender<(NodeId, u64, Fragment)>,
    ) {
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

    fn sender_thread(
        id: NodeId,
        packet_sender: Arc<RwLock<HashMap<u8, Sender<Packet>>>>,
        sim_contr_send: Sender<ServerEvent>,
        fragment_recv: Receiver<(NodeId, u64, Fragment)>,
        nack_recv: Receiver<(NodeId, u64, Fragment)>,
        condv: Condvar,
        ack_packet_buffer: Arc<Mutex<HashMap<(NodeId, u64, u64), Packet>>>,
        topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>,
        topology_modified: Arc<Mutex<bool>>,
    ) {
        // records the routing table for the server (this is only updated when an update to the topology is made)
        let mut routing_table: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        // temporary number
        const MAX_OUTPUT_BUFFER: usize = 10;

        loop {
            // get the ack_buffer through mutex and on condition
            let mut ack_buff = condv
                .wait_while(ack_packet_buffer.lock().unwrap(), |buff| {
                    buff.len() >= MAX_OUTPUT_BUFFER
                })
                .unwrap();

            select_biased!(
                recv(nack_recv) -> nack_res => {
                    if let Ok((destination, session_id, fragment)) = nack_res {
                        // recalculate route if topology was modified
                        let mut topology_modified_lock = topology_modified.lock().unwrap();

                        if *topology_modified_lock {
                            Self::find_route(id, destination, &mut routing_table, topology.clone());
                            *topology_modified_lock = false;
                        }

                        let fragment_index = fragment.fragment_index;

                        let mut packet = Packet {
                            routing_header: SourceRoutingHeader {
                                hops: routing_table.get(&destination).unwrap().clone(),
                                hop_index: 1
                            },
                            session_id,
                            pack_type: PacketType::MsgFragment(fragment) };

                        ack_buff.insert((destination, session_id, fragment_index), packet.clone());

                        Self::send_msg_packet(id, packet_sender.clone(), sim_contr_send.clone(), packet);

                    }
                },
                recv(fragment_recv) -> frag_res => {
                    if let Ok((destination, session_id, fragment)) = frag_res {

                        // receives normal packets

                        // choose the route for the packet
                        if routing_table.get(&destination).is_none() {
                            // if the routing table doesn't have the next hop, then we need to update the routing table
                            Self::find_route(id, destination, &mut routing_table, topology.clone());
                        }

                        let fragment_index = fragment.fragment_index;

                        let mut packet = Packet {
                            routing_header: SourceRoutingHeader {
                                hops: routing_table.get(&destination).unwrap().clone(),
                                hop_index: 1
                            },
                            session_id,
                            pack_type: PacketType::MsgFragment(fragment) };

                        ack_buff.insert((destination, session_id, fragment_index), packet.clone());

                        Self::send_msg_packet(id, packet_sender.clone(), sim_contr_send.clone(), packet);
                    }
                }
            );
        }
    }

    fn message_handler_thread(
        fragment_buffers: Arc<RwLock<HashMap<(NodeId, u64), Vec<Fragment>>>>,
        ready_recv: Receiver<(NodeId, u64)>,
        fragment_send: Sender<(NodeId, u64, Fragment)>,
    ) {
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
                    MessageData::FileMessage {
                        from,
                        to,
                        file,
                        file_name,
                    } => todo!(),
                    MessageData::ResponseClients(items) => todo!(),
                    MessageData::AcknolewdgedAsClient => todo!(),
                    MessageData::ResponseHistory { partner, history } => todo!(),
                    MessageData::UnregisteredSenderError => todo!(),
                    MessageData::UnregisteredRecipientError => todo!(),
                    MessageData::UnsupportedMessageTypeError => todo!(),
                };

                if let Some(msg) = resp_message {
                    let destination_id: NodeId = msg.destination_id;
                    let fragments = Self::disassemble(msg);

                    for fragment in fragments {
                        // send the fragments to the sender thread

                        fragment_send.send((destination_id, todo!(), fragment));
                    }
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
                pack_type: PacketType::Ack(Ack {
                    fragment_index: fragment.fragment_index,
                }),
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
                let mut frag_buffer = fragment_buffers_lock
                    .get_mut(&(packet_source, packet_msg_id))
                    .unwrap();

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
impl ChatServer {
    fn find_route(
        id: NodeId,
        destination: NodeId,
        routing_table: &mut HashMap<NodeId, Vec<NodeId>>,
        topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>,
    ) {
        let topology_lock = topology.read().unwrap();

        // use path finding algorithm to find the route
        let path = algo::astar(
            &*topology_lock,
            id,
            |finish| finish == destination,
            |_| 1,
            |_| 0,
        );

        if let Some((_, route)) = path {
            routing_table.insert(destination, route);
        }
    }

    fn send_msg_packet(
        id: NodeId,
        packet_sender: Arc<RwLock<HashMap<u8, Sender<Packet>>>>,
        sim_contr_send: Sender<ServerEvent>,
        packet: Packet,
    ) {
        let next_node = packet.routing_header.hops[packet.routing_header.hop_index];
        let send_channel = &packet_sender.read().unwrap()[&next_node];

        log::debug!(
            "{} from {} - packet: {}",
            " -> packet sent ".blue(),
            id,
            packet
        );

        let res = send_channel.send(packet.clone());

        if let Err(mut packet) = res {
            log::error!("The send inside channel gave an error, this shouldn't be happening");
        } else {
            sim_contr_send.send(ServerEvent::PacketSent(packet));
        }
    }
}

// Message handler thread functions
impl ChatServer {}
