use std::{
    cell::RefCell,
    collections::{HashMap, HashSet, VecDeque},
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
    client,
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
    ack_packet_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>, // stores packets that need to await an ack. The tuple is (session_id, frag_index)
    topology_modified: Arc<Mutex<bool>>, // flag to check if the topology has been modified
    edge_nodes: Arc<RwLock<HashSet<NodeId>>>, // stores the edge nodes that can't be used in a route
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
        Self {
            id,
            sim_contr_send,
            sim_contr_recv,
            packet_recv,
            packet_send: Arc::new(RwLock::new(packet_send)),
            flood_id: 0,
            topology: Arc::new(RwLock::new(GraphMap::new())),
            fragment_buffers: Arc::new(RwLock::new(HashMap::new())),
            ack_packet_buffer: Arc::new(Mutex::new(HashMap::new())),
            topology_modified: Arc::new(Mutex::new(false)),
            edge_nodes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    fn run(&mut self) {
        // thread handles for all threads spawned by this server
        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        // spawn the sender thread, clone all the necessary channels for thread spawn
        let packet_sender = self.packet_send.clone();
        let sim_contr_sender = self.sim_contr_send.clone();

        let (nack_send, nack_recv) = unbounded::<Packet>();

        let (fragment_send, fragment_recv) = unbounded::<(NodeId, u64, Fragment)>();

        let condv = Condvar::new();

        let ack_packet_buffer = self.ack_packet_buffer.clone();

        let topology = self.topology.clone();

        let id = self.id;

        let topology_modified = self.topology_modified.clone();

        let edge_nodes = self.edge_nodes.clone();

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
                edge_nodes,
            );
        }));

        // spawn the message handling thread
        let fragment_buffers = self.fragment_buffers.clone();

        // ready channel to start the assembly of the fragments
        let (ready_send, ready_recv) = unbounded::<(NodeId, u64)>();

        let id = self.id;

        threads.push(thread::spawn(move || {
            ChatServer::message_handler_thread(id, fragment_buffers, ready_recv, fragment_send);
        }));

        // at the end call the receiver thread (which is this one)
        self.receiver_thread(ready_send, nack_send);
    }
}

impl Fragmenter for ChatServer {
    fn assemble(mut fragments: Vec<Fragment>) -> Message {
        // sort fragments by index before assembling
        fragments.sort_by(|a, b| a.fragment_index.cmp(&b.fragment_index));

        let mut message_data: Vec<u8> = Vec::new();
        for fragment in fragments {
            if fragment.length < 128 {
                message_data.extend(&fragment.data[0..fragment.length as usize]);
            } else {
                message_data.extend(&fragment.data);
            }
        }

        Message::from_u8(message_data)
    }

    fn disassemble(msg: Message) -> std::collections::VecDeque<Fragment> {
        let mut message_data = msg.into_u8();

        message_data.reverse();

        let mut fragments: VecDeque<Fragment> = VecDeque::new();
        let frag_numbers = (message_data.len() as f64 / 128.0).ceil() as u64;

        for i in 0..frag_numbers {
            let mut fragment_data: [u8; 128] = [0; 128];
            let mut lenght: u8 = 0;
            for index in 0..128 {
                if let Some(byte) = message_data.pop() {
                    fragment_data[index] = byte;
                    lenght += 1;
                } else {
                    break;
                }
            }

            fragments.push_back(Fragment {
                fragment_index: i as u64,
                total_n_fragments: frag_numbers,
                length: lenght,
                data: fragment_data,
            });
        }
        fragments
    }
}

impl ChatServer {
    fn receiver_thread(&mut self, ready_send: Sender<(NodeId, u64)>, nack_send: Sender<Packet>) {
        loop {
            select_biased!(
                recv(self.sim_contr_recv) -> cmd => {
                    if let Ok(command) = cmd {
                        match command {
                            ServerCommand::NetworkInitialized => self.initiate_flood(),
                            ServerCommand::AddSender(id, sender) => self.add_sender(id, sender),
                            ServerCommand::RemoveSender(id) => self.remove_sender(id),
                        }
                    }
                },
                recv(self.packet_recv) -> res => {
                    if let Ok(mut packet) = res {
                        // send packet to the simulation controller
                        self.sim_contr_send.send(ServerEvent::PacketReceived(packet.clone()));

                        // match the packet type and act accordingly
                        match packet.pack_type {
                            PacketType::MsgFragment(_) => self.manage_msg_fragment(packet, ready_send.clone()),
                            PacketType::Ack(ack) => self.manage_ack(packet.session_id, ack),
                            PacketType::Nack(nack) => self.manage_nack(packet.session_id, nack, nack_send.clone()),
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
        nack_recv: Receiver<Packet>,
        condv: Condvar,
        ack_packet_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>,
        topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>,
        topology_modified: Arc<Mutex<bool>>,
        edge_nodes: Arc<RwLock<HashSet<NodeId>>>,
    ) {
        // records the routing table for the server (this is only updated when an update to the topology is made)
        let mut routing_table: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        // temporary number
        const MAX_OUTPUT_BUFFER: usize = 10;

        loop {
            select_biased!(
                recv(nack_recv) -> nack_res => {
                    if let Ok(mut packet) = nack_res {
                        // recalculate route if topology was modified
                        let mut topology_modified_lock = topology_modified.lock().unwrap();

                        let destination = packet.routing_header.hops.last().unwrap().clone();

                        if *topology_modified_lock {
                            Self::find_route(id, destination, &mut routing_table, topology.clone(), edge_nodes.clone());
                            *topology_modified_lock = false;
                        }

                        packet.routing_header.hops = routing_table.get(&destination).unwrap().clone();

                        // get the ack_buffer through mutex and on condition
                        let mut ack_buff = condv
                        .wait_while(ack_packet_buffer.lock().unwrap(), |buff| {
                            buff.len() + nack_recv.len() >= MAX_OUTPUT_BUFFER
                        })
                        .unwrap();

                        if let PacketType::MsgFragment(fragment) = &packet.pack_type {
                            ack_buff.insert((packet.session_id, fragment.fragment_index), packet.clone());
                        }

                        Self::send_msg_packet(id, packet_sender.clone(), sim_contr_send.clone(), packet);

                    }
                },
                recv(fragment_recv) -> frag_res => {
                    if let Ok((destination, session_id, fragment)) = frag_res {

                        // receives normal packets

                        // choose the route for the packet
                        if routing_table.get(&destination).is_none() {
                            // if the routing table doesn't have the next hop, then we need to update the routing table
                            Self::find_route(id, destination, &mut routing_table, topology.clone(), edge_nodes.clone());
                        }

                        let fragment_index = fragment.fragment_index;

                        let mut packet = Packet {
                            routing_header: SourceRoutingHeader {
                                hops: routing_table.get(&destination).unwrap().clone(),
                                hop_index: 1
                            },
                            session_id,
                            pack_type: PacketType::MsgFragment(fragment) };

                        // get the ack_buffer through mutex and on condition
                        let mut ack_buff = condv
                        .wait_while(ack_packet_buffer.lock().unwrap(), |buff| {
                            buff.len() + nack_recv.len() >= MAX_OUTPUT_BUFFER
                        })
                        .unwrap();

                        ack_buff.insert((session_id, fragment_index), packet.clone());

                        Self::send_msg_packet(id, packet_sender.clone(), sim_contr_send.clone(), packet);
                    }
                }
            );
        }
    }

    fn message_handler_thread(
        id: NodeId,
        fragment_buffers: Arc<RwLock<HashMap<(NodeId, u64), Vec<Fragment>>>>,
        ready_recv: Receiver<(NodeId, u64)>,
        fragment_send: Sender<(NodeId, u64, Fragment)>,
    ) {
        let mut client_table: HashSet<NodeId> = HashSet::new();
        // history has the nodes ordered in ascending order, i.e. the first NodeId is lower than the second
        let mut history_table: HashMap<(NodeId, NodeId), Vec<ChatMessage>> = HashMap::new();
        // stores the latest session id
        let mut session_id = 1;

        // basically infinite loop waiting for ready signal from the receiver thread
        while let Ok((source, msg_id)) = ready_recv.recv() {
            let mut fragment_buffers_lock = fragment_buffers.write().unwrap();

            // assemble the message
            if let Some(fragments) = fragment_buffers_lock.remove(&(source, msg_id)) {
                let mut message = Self::assemble(fragments);

                log::debug!(
                    "{} {} assembled message: {:?}",
                    "↳ server".green(),
                    id,
                    message
                );

                // check the message type and act accordingly
                let mut resp_message: Option<Message> = match message.message_data {
                    MessageData::RegisterAsClient(client) => {
                        Self::register_client(&mut client_table, client, id)
                    }
                    MessageData::UnregisterAsClient(client) => {
                        Self::unregister_client(&mut client_table, client)
                    }
                    MessageData::RequestClients(destination) => {
                        Self::request_clients(&client_table, id, destination)
                    }
                    MessageData::RequestHistory { requester, partner } => {
                        Self::request_history(requester, partner, id, &history_table)
                    }
                    MessageData::TextMessage { from, to, text } => {
                        if !client_table.contains(&to) {
                            Self::error_msg(id, from, MessageData::UnregisteredRecipientError)
                        } else if !client_table.contains(&from) {
                            Self::error_msg(id, from, MessageData::UnregisteredSenderError)
                        } else {
                            Self::text_message(from, to, text, id, &mut history_table)
                        }
                    }
                    MessageData::FileMessage {
                        from,
                        to,
                        file,
                        file_name,
                    } => {
                        if !client_table.contains(&to) {
                            Self::error_msg(id, from, MessageData::UnregisteredRecipientError)
                        } else if !client_table.contains(&from) {
                            Self::error_msg(id, from, MessageData::UnregisteredSenderError)
                        } else {
                            Self::file_message(from, to, file, file_name, id, &mut history_table)
                        }
                    }
                    MessageData::ResponseClients(items) => {
                        Self::error_msg(id, source, MessageData::UnsupportedMessageTypeError)
                    }
                    MessageData::AcknolewdgedAsClient => {
                        Self::error_msg(id, source, MessageData::UnsupportedMessageTypeError)
                    }
                    MessageData::ResponseHistory { partner, history } => {
                        Self::error_msg(id, source, MessageData::UnsupportedMessageTypeError)
                    }
                    MessageData::UnregisteredSenderError => None,
                    MessageData::UnregisteredRecipientError => None,
                    MessageData::UnsupportedMessageTypeError => None,
                };

                if let Some(msg) = resp_message {
                    let destination_id: NodeId = msg.destination_id;
                    let fragments = Self::disassemble(msg);

                    for fragment in fragments {
                        // send the fragments to the sender thread
                        fragment_send.send((destination_id, session_id, fragment));
                    }
                    session_id += 1;
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
            let mut edge_nodes_lock = self.edge_nodes.write().unwrap();
            let mut topology_modified_lock = self.topology_modified.lock().unwrap();

            let mut current_index = topology_lock.add_node(self.id);

            for (node_id, node_type) in fr.path_trace.iter().skip(1) {
                let next_index = topology_lock.add_node(*node_id);
                // add the edge nodes to the edge_nodes set
                match node_type {
                    NodeType::Client => {
                        edge_nodes_lock.insert(*node_id);
                    }
                    NodeType::Drone => {}
                    NodeType::Server => {
                        edge_nodes_lock.insert(*node_id);
                    }
                }

                topology_lock.add_edge(current_index, next_index, ());
                current_index = next_index;
            }
            *topology_modified_lock = true;
        }

        log::info!("{} {:?}", "Server topology: ".green(), self.topology);
    }

    fn manage_msg_fragment(&self, packet: Packet, ready_send: Sender<(NodeId, u64)>) {
        log::debug!(
            "{} {} received a message fragment: {}",
            "↳ server".green(),
            self.id,
            packet
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

    fn manage_ack(&self, session_id: u64, ack: Ack) {
        log::debug!(
            "{} {} received an ack: {:?}",
            "↳ server".green(),
            self.id,
            ack
        );

        let ack_key = (session_id, ack.fragment_index);

        let mut ack_packet_buffer_lock = self.ack_packet_buffer.lock().unwrap();

        if let Some(packet) = ack_packet_buffer_lock.remove(&ack_key) {
            log::debug!(
                "{} {} removed packet from ack buffer: {:?}",
                "↳ server".green(),
                self.id,
                packet
            );
        }
    }

    fn manage_nack(&self, session_id: u64, nack: Nack, nack_send: Sender<Packet>) {
        log::debug!(
            "{} {} received a nack: {:?}",
            "↳ server".green(),
            self.id,
            nack
        );

        // fast return flag to throw the packet away if a weird error happens
        let mut fast_return = false;

        match &nack.nack_type {
            packet::NackType::ErrorInRouting(node) => {
                self.topology.write().unwrap().remove_node(*node);
                *self.topology_modified.lock().unwrap() = true;
            }
            packet::NackType::DestinationIsDrone => {
                log::error!("The destination is a drone, this shouldn't be happening");
                fast_return = true;
            }
            packet::NackType::Dropped => {
                // do nothing for now, maybe in the future update the edge weights
            }
            packet::NackType::UnexpectedRecipient(_) => {
                log::error!("The recipient is not the expected one, this shouldn't be happening");
                fast_return = true;
            }
        }

        let ack_key = (session_id, nack.fragment_index);

        let mut ack_packet_buffer_lock = self.ack_packet_buffer.lock().unwrap();

        if let Some(packet) = ack_packet_buffer_lock.remove(&ack_key) {
            if fast_return {
                return;
            }
            nack_send.send(packet);
        } else {
            log::error!("The packet was not found in the ack buffer, this shouldn't be happening");
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

    fn initiate_flood(&mut self) {
        for (id, sender) in self.packet_send.read().unwrap().iter() {
            let packet = Packet {
                pack_type: PacketType::FloodRequest(FloodRequest {
                    path_trace: vec![(self.id, NodeType::Server)],
                    flood_id: self.flood_id,
                    initiator_id: self.id,
                }),
                routing_header: SourceRoutingHeader {
                    hops: vec![],
                    hop_index: 0,
                },
                session_id: 0, // it'll be whatever for now
            };
            self.flood_id += 1;

            let res = sender.send(packet.clone());

            if let Err(mut packet) = res {
                log::error!("The send inside channel gave an error, this shouldn't be happening");
            } else {
                self.sim_contr_send.send(ServerEvent::PacketSent(packet));
            }
        }
    }

    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.write().unwrap().insert(id, sender);
    }

    fn remove_sender(&mut self, id: NodeId) {
        self.packet_send.write().unwrap().remove(&id);
    }
}

// Sender thread functions
impl ChatServer {
    fn find_route(
        id: NodeId,
        destination: NodeId,
        routing_table: &mut HashMap<NodeId, Vec<NodeId>>,
        topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>,
        edge_nodes: Arc<RwLock<HashSet<NodeId>>>,
    ) {
        let topology_lock = topology.read().unwrap();

        // use path finding algorithm to find the route
        let path = algo::astar(
            &*topology_lock,
            id,
            |finish| finish == destination,
            |(a, b, _)| {
                if destination == a || destination == b {
                    return 1;
                }
                // if a node is an edge node, then the weight should be "infinite" as it can't be used
                let edge_nodes_lock = edge_nodes.read().unwrap();
                if edge_nodes_lock.contains(&b) || edge_nodes_lock.contains(&a) {
                    topology_lock.edge_count()
                } else {
                    1
                }
            },
            |_| 0,
        );

        if let Some((_, route)) = path {
            routing_table.insert(destination, route);
        } else {
            log::error!("No route found to destination {}", destination);
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
impl ChatServer {
    fn register_client(
        client_table: &mut HashSet<NodeId>,
        client: NodeId,
        id: NodeId,
    ) -> Option<Message> {
        client_table.insert(client);

        Some(Message::new(id, client, MessageData::AcknolewdgedAsClient))
    }

    fn unregister_client(client_table: &mut HashSet<NodeId>, client: NodeId) -> Option<Message> {
        client_table.remove(&client);
        None
    }

    fn request_clients(
        client_table: &HashSet<NodeId>,
        id: NodeId,
        destination: NodeId,
    ) -> Option<Message> {
        let clients: Vec<NodeId> = client_table.iter().cloned().collect();

        Some(Message::new(
            id,
            destination,
            MessageData::ResponseClients(clients),
        ))
    }

    fn request_history(
        requester: NodeId,
        partner: NodeId,
        id: NodeId,
        history_table: &HashMap<(NodeId, NodeId), Vec<ChatMessage>>,
    ) -> Option<Message> {
        // order the nodes in ascending order to keep the history consistent
        let mut key_tuple: (NodeId, NodeId) = (requester, partner);
        if partner < requester {
            key_tuple = (partner, requester);
        }

        let history = history_table.get(&key_tuple)?.clone();

        Some(Message::new(
            id,
            requester,
            MessageData::ResponseHistory { partner, history },
        ))
    }

    fn text_message(
        from: NodeId,
        to: NodeId,
        text: String,
        id: NodeId,
        history_table: &mut HashMap<(NodeId, NodeId), Vec<ChatMessage>>,
    ) -> Option<Message> {
        let message = Message::new(
            id,
            to,
            MessageData::TextMessage {
                from,
                to,
                text: text.clone(),
            },
        );

        // order the nodes in ascending order to keep the history consistent
        let mut key_tuple: (NodeId, NodeId) = (from, to);
        if to < from {
            key_tuple = (to, from);
        }

        // add the message to the history table
        history_table
            .entry(key_tuple)
            .or_insert_with(Vec::new)
            .push(ChatMessage::TextMessage { from, to, text });

        Some(message)
    }

    fn file_message(
        from: NodeId,
        to: NodeId,
        file: Vec<u8>,
        file_name: String,
        id: NodeId,
        history_table: &mut HashMap<(NodeId, NodeId), Vec<ChatMessage>>,
    ) -> Option<Message> {
        let message = Message::new(
            id,
            to,
            MessageData::FileMessage {
                from,
                to,
                file: file.clone(),
                file_name: file_name.clone(),
            },
        );

        // order the nodes in ascending order to keep the history consistent
        let mut key_tuple: (NodeId, NodeId) = (from, to);
        if to < from {
            key_tuple = (to, from);
        }

        // add the message to the history table
        history_table
            .entry(key_tuple)
            .or_insert_with(Vec::new)
            .push(ChatMessage::FileMessage {
                from,
                to,
                file,
                file_name,
            });

        Some(message)
    }

    fn error_msg(id: NodeId, destination: NodeId, error_type: MessageData) -> Option<Message> {
        Some(Message::new(id, destination, error_type))
    }
}
