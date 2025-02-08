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
use eframe::glow::PACK_ROW_LENGTH;
use egui::accesskit::Node;
use petgraph::{
    algo,
    data::Build,
    prelude::{GraphMap, StableGraph},
    Undirected,
};
use serde::de::DeserializeSeed;
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{
        self, Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType,
        FRAGMENT_DSIZE,
    },
};

use crate::{
    client,
    fragmentation::{
        self,
        message::{self, ChatMessage, Message, MessageData},
        Fragmenter,
    },
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

use super::ClientTrait;

pub struct ClientLeonardo {
    id: NodeId,
    flood_id: u64,
    chat_server_id: NodeId,

    //channels with sim controller
    sim_contr_send: Sender<ClientEvent>,
    sim_contr_recv: Receiver<ClientCommand>,
    //channels with drones
    packet_recv: Receiver<Packet>,
    packet_send: Arc<RwLock<HashMap<NodeId, Sender<Packet>>>>,

    //topology of the net, weight depends on prd
    topology: Arc<RwLock<GraphMap<NodeId, (f64), Undirected>>>,

    //buffers for the fragments
    fragment_buffer: Arc<RwLock<HashMap<(NodeId, u64), Vec<Fragment>>>>,
    ack_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>,
    topology_modified: Arc<Mutex<bool>>,
    edge_nodes: Arc<RwLock<HashSet<NodeId>>>,

    //chat history with other clients
    history: HashMap<NodeId, Vec<ChatMessage>>,
}

impl ClientTrait for ClientLeonardo {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ClientEvent>,
        sim_contr_recv: Receiver<ClientCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
    ) -> Self
    where
        Self: Sized,
    {
        ClientLeonardo {
            id,
            flood_id: 0,
            chat_server_id: 0,
            sim_contr_send,
            sim_contr_recv,
            packet_recv,
            packet_send: Arc::new(RwLock::new(packet_send)),
            topology: Arc::new(RwLock::new(GraphMap::new())),
            fragment_buffer: Arc::new(RwLock::new(HashMap::new())),
            ack_buffer: Arc::new(Mutex::new(HashMap::new())),
            topology_modified: Arc::new(Mutex::new(false)),
            edge_nodes: Arc::new(RwLock::new(HashSet::new())),
            history: HashMap::new(),
        }
    }

    fn run(&mut self) {
        //cloning all the parameters for each thread

        let (ready_for_receiver, ready_for_handler) = unbounded::<(NodeId, u64)>();
        let (thread_sender, thread_receiver) = unbounded::<Message>();
        let (fragment_sender, fragment_receiver) = unbounded::<(NodeId, u64, Fragment)>();

        let (nacks, nacks_recv) = unbounded::<Packet>();

        let id = self.id;
        let packet_send = self.packet_send.clone();
        let sim_contr_send = self.sim_contr_send.clone();
        let fragment_buffer = self.fragment_buffer.clone();
        let ack_buffer = self.ack_buffer.clone();
        let topology = self.topology.clone();
        let edge_nodes = self.edge_nodes.clone();
        let condv = Condvar::new();

        thread::spawn(move || {
            Self::sender_thread(
                id,
                packet_send,
                sim_contr_send,
                fragment_receiver,
                nacks_recv,
                Condvar::new(),
                ack_buffer,
                topology,
                edge_nodes,
            );
        });

        let id = self.id;
        let fragment_buffer = self.fragment_buffer.clone();
        let ready_for_handler = ready_for_handler.clone();
        let thread_receiver = thread_receiver.clone();
        let sim_control_send = self.sim_contr_send.clone();
        let fragment_sender = fragment_sender.clone();

        thread::spawn(move || {
            Self::message_handler_thread(
                id,
                fragment_buffer,
                ready_for_handler,
                thread_receiver,
                fragment_sender,
                sim_control_send,
            );
        });

        self.receiver_thread(ready_for_receiver, thread_sender, nacks);
    }
}

// ---------------Threads Functions-----------------//

impl ClientLeonardo {
    /*
            Using 3 different threads:
            - Sender thread: calculates routes and sends packets
            - Receiver thread: receives packets and sends simple messages (acks, floods, etc), also manages the buffers
            - Controller thread: receives commands from the simulation controller and sends events to it

    */
    fn receiver_thread(
        &mut self,
        ready: Sender<(NodeId, u64)>,
        thread_sender: Sender<Message>,
        nacks: Sender<Packet>,
    ) {
        loop {
            select_biased!(
                recv(self.sim_contr_recv) -> msg => {
                    if let Ok(c) = msg {
                        match c {
                            ClientCommand::StartFlooding => {
                                self.initiate_flood();
                            },
                            ClientCommand::AddSender(id, sender) => {
                                self.add_sender(id, sender);
                            },
                            ClientCommand::RemoveSender(id) => {
                                self.remove_sender(id);
                            },
                            ClientCommand::GetResponseClient => {
                                self.get_response_clients(thread_sender.clone());
                            },
                            ClientCommand::RegisterAsClient => {
                                self.register_as_client(thread_sender.clone());
                            },
                            ClientCommand::UnregisterAsClient => {
                                self.unregister_as_client(thread_sender.clone());
                            },
                            ClientCommand::OpenChatWith(partner) => {
                                let history = self.history.get(&partner).cloned().unwrap_or_default();
                                self.get_response_history(partner, history, thread_sender.clone());
                            },
                            ClientCommand::SendTextMessageTo{receiver, message} => {
                                self.send_text_message_to(receiver, message, thread_sender.clone());
                            },
                            ClientCommand::SendFileMessageTo{receiver, file} => {
                                todo!();
                            },
                        }
                    }
                },
                recv(self.packet_recv) -> packet => {

                    if let Ok(p) = packet {
                        let header_vec = p.routing_header.hops.clone();
                        match p.pack_type {
                            PacketType::Ack(ack) => {
                                self.manage_ack(header_vec, ack, p.session_id);
                            },
                            PacketType::Nack(nack) => {
                                self.manage_nack(header_vec, nack, p.session_id, nacks.clone());
                            },
                            PacketType::FloodRequest(_) => {
                                self.manage_flood_request(p);
                            },
                            PacketType::FloodResponse(flood_res) => {
                                self.manage_flood_response(flood_res);
                            },
                            PacketType::MsgFragment(_) => {
                                self.manage_msg_fragment(p, ready.clone());
                            },
                            _ => {}
                        }
                    }
                }
            )
        }
    }

    fn sender_thread(
        id: NodeId,
        packet_sender: Arc<RwLock<HashMap<u8, Sender<Packet>>>>,
        sim_contr_send: Sender<ClientEvent>,
        fragment_recv: Receiver<(NodeId, u64, Fragment)>,
        nack_recv: Receiver<Packet>,
        condv: Condvar,
        ack_packet_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>,
        topology: Arc<RwLock<GraphMap<NodeId, f64, Undirected>>>,
        edge_nodes: Arc<RwLock<HashSet<NodeId>>>,
    ) {
        // temporary number
        const MAX_OUTPUT_BUFFER: usize = 10;

        loop {
            select_biased!(
                recv(nack_recv) -> nack_res => {
                    if let Ok(mut packet) = nack_res {
                        // recalculate route if topology was modified

                        let destination = packet.routing_header.hops.last().unwrap().clone();

                        //condv limits the size of ack_buffer by freezing the thread until condition is reached
                        let mut ack_buff = condv.wait_while(ack_packet_buffer.lock().unwrap(), |buff| {
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

                        // if the routing table doesn't have the next hop, then we need to update the routing table
                        let route = Self::find_route(id, destination, edge_nodes.clone(), topology.clone());


                        let fragment_index = fragment.fragment_index;

                        let mut packet = Packet {
                            routing_header: SourceRoutingHeader {
                                hops: route,
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
        ready: Receiver<(NodeId, u64)>,
        command_recv: Receiver<Message>,
        fragment_sender: Sender<(NodeId, u64, Fragment)>,
        sim_send: Sender<ClientEvent>,
    ) {
        let mut session_id = 1;

        loop {
            select_biased!(
                recv(command_recv) -> frag_res => {
                    //disassembly of the message and send to sender thread

                    let fragments = Self::disassemble(frag_res.unwrap());

                    for fragment in fragments {
                        fragment_sender.send((id, session_id, fragment));
                    }

                    //increment session_id
                    session_id += 1;

                },
                recv(ready) -> ready_res => {
                    if let Ok((source, session_id)) = ready_res {
                        let fragment_buffer_lock = fragment_buffers.read().unwrap();
                        let buffer = fragment_buffer_lock.get(&(source, session_id)).unwrap().clone();
                        let message = Self::assemble(buffer);


                        match message.message_data {
                            MessageData::ResponseClients(res) => {
                                Self::send_response(res, sim_send.clone());
                            },
                            MessageData::AcknolewdgedAsClient => {
                                Self::client_ack(sim_send.clone());
                            },
                            MessageData::ResponseHistory{partner, history} => {
                                Self::response_history(partner, history, sim_send.clone());
                            },
                            MessageData::UnregisteredSenderError => {
                                Self::unregistered_sender_error(sim_send.clone());
                            },
                            MessageData::UnregisteredRecipientError => {
                                Self::unregistered_recipient_error(sim_send.clone());
                            },
                            MessageData::UnsupportedMessageTypeError => {
                                Self::unsupported_message_type_error(sim_send.clone());
                            },
                            MessageData::TextMessage{from, to, text} => {
                                Self::text_message_received(from, to, text, sim_send.clone());
                            },
                            _ => {}

                        }
                    }
                }
            );
        }
    }
}

//-----------------Function Implementations-----------------//

impl ClientLeonardo {
    //-------------------Sender Thread-------------------//

    fn find_route(
        start_id: NodeId,
        destination_id: NodeId,
        avoid_nodes: Arc<RwLock<HashSet<NodeId>>>,
        topology: Arc<RwLock<GraphMap<NodeId, f64, Undirected>>>,
    ) -> Vec<NodeId> {
        let topology_lock = topology.read().unwrap();

        let path = algo::astar(
            &*topology_lock,
            start_id,
            |end| end == destination_id,
            |(a, b, weight)| {
                if (destination_id == b || destination_id == a) {
                    return 1;
                }
                let avoid_nodes_lock = avoid_nodes.read().unwrap();
                if avoid_nodes_lock.contains(&a) || avoid_nodes_lock.contains(&b) {
                    return topology_lock.edge_count();
                } else {
                    1
                }
            },
            |_| 0,
        );

        path.unwrap().1
    }

    fn send_msg_packet(
        id: NodeId,
        packet_sender: Arc<RwLock<HashMap<u8, Sender<Packet>>>>,
        sim_contr_send: Sender<ClientEvent>,
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
            sim_contr_send.send(ClientEvent::PacketSent(packet));
        }
    }

    //------------------Receiver Thread------------------//

    fn manage_ack(&self, header_vec: Vec<NodeId>, ack: Ack, s_id: u64) {
        let mut ack_buffer_lock = self.ack_buffer.lock().unwrap();
        let key = (s_id, ack.fragment_index);

        //if receiving ack then removing it from the the ack buffer
        if let Some(p) = ack_buffer_lock.remove(&key) {
            log::debug!(
                "{} Ack received for fragment {} of message {}",
                "↳ client".purple(),
                ack.fragment_index,
                s_id
            );
        } else {
            log::error!(
                "{} Ack received for fragment {} of message {} but it was not found in the buffer",
                "↳ client".purple(),
                ack.fragment_index,
                s_id
            );
        }

        for i in 0..header_vec.len() - 1 {
            let h1 = header_vec[i];
            let h2 = header_vec[i + 1];
            let mut topology_lock = self.topology.write().unwrap();
            let mut curr_weight = *topology_lock.edge_weight(h1, h2).unwrap();
            topology_lock.update_edge(h1, h2, (curr_weight * 0.60) - 0.40);
        }
    }

    fn manage_nack(&self, header_vec: Vec<NodeId>, nack: Nack, s_id: u64, resend: Sender<Packet>) {
        match &nack.nack_type {
            packet::NackType::ErrorInRouting(n) => {
                self.topology.write().unwrap().remove_node(*n);
                *self.topology_modified.lock().unwrap() = true;
                log::error!("{} Error in routing {}", "↳ client".purple(), n);
            }
            packet::NackType::DestinationIsDrone => {
                log::error!(
                    "{} Destination is a drone: {} {}",
                    "↳ client".purple(),
                    nack.fragment_index,
                    s_id
                );
                return;
            }
            packet::NackType::Dropped => {
                for i in 0..header_vec.len() - 1 {
                    let h1 = header_vec[i];
                    let h2 = header_vec[i + 1];
                    let mut topology_lock = self.topology.write().unwrap();
                    let mut curr_weight = *topology_lock.edge_weight(h1, h2).unwrap();
                    topology_lock.update_edge(h1, h2, (curr_weight * 0.60) + 0.40);
                }
            }
            packet::NackType::UnexpectedRecipient(_) => {
                log::error!("{} Unexpected recipient", "↳ client".purple());
            }
        }

        let mut ack_buffer_lock = self.ack_buffer.lock().unwrap();
        let key = (s_id, nack.fragment_index);

        if let Some(p) = ack_buffer_lock.remove(&key) {
            log::debug!(
                "{} Nack received for fragment {} of message {}",
                "↳ client".purple(),
                nack.fragment_index,
                s_id
            );
            resend.send(p);
        } else {
            log::error!(
                "{} Nack received for fragment {} of message {} but it was not found in the buffer",
                "↳ client".red(),
                nack.fragment_index,
                s_id
            );
        }
    }

    fn manage_flood_request(&self, p: Packet) {
        log::debug!(
            "{} {} Flood request received: {}",
            "↳ client".purple(),
            self.id,
            p.pack_type
        );

        if let PacketType::FloodRequest(mut flood) = p.pack_type {
            flood.path_trace.push((self.id, NodeType::Client));

            let flood_res = FloodResponse {
                path_trace: flood.path_trace,
                flood_id: flood.flood_id,
            };

            let mut route: Vec<NodeId> = flood_res.path_trace.iter().map(|(id, _)| *id).collect();
            route.reverse();

            let packet = Packet {
                routing_header: SourceRoutingHeader {
                    hops: route,
                    hop_index: 1,
                },
                session_id: 0,
                pack_type: PacketType::FloodResponse(flood_res),
            };

            self.send_packet(packet);
        }
    }

    fn manage_flood_response(&mut self, flood_res: FloodResponse) {
        log::debug!(
            "{} {} Flood response received: {}",
            "↳ client".purple(),
            self.id,
            flood_res
        );

        let mut topology_lock = self.topology.write().unwrap();
        let mut edge_nodes_lock = self.edge_nodes.write().unwrap();
        let mut topology_modified_lock = self.topology_modified.lock().unwrap();
        let mut index = topology_lock.add_node(self.id);

        for (id, node_type) in flood_res.path_trace.iter() {
            let next = topology_lock.add_node(*id);
            match node_type {
                NodeType::Client => {
                    edge_nodes_lock.insert(*id);
                }
                NodeType::Server => {
                    edge_nodes_lock.insert(*id);
                    self.chat_server_id = *id;
                }
                _ => {}
            }

            //inizializza tutti i weight a 1
            topology_lock.add_edge(index, next, 0.0);
        }

        *topology_modified_lock = true;
    }

    //sender to the the message handling
    fn manage_msg_fragment(&self, p: Packet, ready: Sender<(NodeId, u64)>) {
        log::debug!(
            "{} {} Message fragment received: {}",
            "↳ client".purple(),
            self.id,
            p.pack_type
        );

        let p_source = p.routing_header.hops[0];
        let id = p.session_id;

        // generating ack route
        let mut route = p.routing_header.hops.clone();
        route.reverse();

        if let PacketType::MsgFragment(f) = p.pack_type {
            //sending the Ack
            let ack = Packet {
                pack_type: PacketType::Ack(Ack {
                    fragment_index: f.fragment_index,
                }),
                routing_header: SourceRoutingHeader {
                    hops: route,
                    hop_index: 1,
                },
                session_id: id,
            };

            self.send_packet(ack);

            //update fragment buffer and manage the cases

            let mut fragment_buffer_lock = self.fragment_buffer.write().unwrap();
            let tot = f.total_n_fragments;

            //could receive multiple messages at the same time, so the map store different buffers
            if fragment_buffer_lock.contains_key(&(p_source, id)) {
                let mut buffer = fragment_buffer_lock.get_mut(&(p_source, id)).unwrap();
                buffer.push(f);

                if buffer.len() == tot as usize {
                    ready.send((p_source, id));
                }
            } else {
                fragment_buffer_lock.insert((p_source, id), vec![f]);
                if tot == 1 {
                    ready.send((p_source, id));
                }
            }
        }
    }

    fn send_packet(&self, packet: Packet) {
        let next = packet.routing_header.hops[packet.routing_header.hop_index];
        let send_channel = &self.packet_send.read().unwrap()[&next]; //no need for arc and mutex

        //standard log
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
            self.sim_contr_send.send(ClientEvent::PacketSent(packet));
        }
    }

    //-----------------Controller Thread-----------------//

    //events to simulation controller

    fn send_response(res: Vec<u8>, sim_send: Sender<ClientEvent>) {
        sim_send.send(ClientEvent::ResponseClientsReceived(res));
    }

    fn client_ack(sim_send: Sender<ClientEvent>) {
        sim_send.send(ClientEvent::AcknolewdgedAsClient);
    }

    fn response_history(partner: NodeId, history: Vec<ChatMessage>, sim_send: Sender<ClientEvent>) {
        sim_send.send(ClientEvent::ResponseHistoryReceived { partner, history });
    }

    fn unregistered_sender_error(sim_send: Sender<ClientEvent>) {
        sim_send.send(ClientEvent::UnregisteredSenderError);
    }

    fn unregistered_recipient_error(sim_send: Sender<ClientEvent>) {
        sim_send.send(ClientEvent::UnregisteredRecipientError);
    }

    fn unsupported_message_type_error(sim_send: Sender<ClientEvent>) {
        sim_send.send(ClientEvent::UnsupportedMessageTypeError);
    }

    fn text_message_received(
        from: NodeId,
        to: NodeId,
        text: String,
        sim_send: Sender<ClientEvent>,
    ) {
        sim_send.send(ClientEvent::TextMessage { from, to, text });
    }
    //commands from simulation controller

    fn initiate_flood(&mut self) {
        for (id, recv) in self.packet_send.read().unwrap().iter() {
            let packet = Packet {
                routing_header: SourceRoutingHeader {
                    hops: vec![],
                    hop_index: 1,
                },
                session_id: 0,
                pack_type: PacketType::FloodRequest(FloodRequest {
                    path_trace: vec![(self.id, NodeType::Client)],
                    flood_id: self.flood_id,
                    initiator_id: self.id,
                }),
            };

            self.flood_id += 1;

            let res = recv.send(packet.clone());

            if let Err(mut packet) = res {
                log::error!("The send inside channel gave an error, this shouldn't be happening");
            } else {
                self.sim_contr_send.send(ClientEvent::PacketSent(packet));
            }
        }
    }

    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_send.write().unwrap().insert(id, sender);
    }

    fn remove_sender(&mut self, id: NodeId) {
        self.packet_send.write().unwrap().remove(&id);
    }

    fn get_response_clients(&self, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            self.chat_server_id,
            MessageData::RequestClients(self.id),
        );
        sender.send(m);
    }

    fn register_as_client(&self, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            self.chat_server_id,
            MessageData::RegisterAsClient(self.id),
        );
        sender.send(m);
    }

    fn unregister_as_client(&self, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            self.chat_server_id,
            MessageData::UnregisterAsClient(self.id),
        );
        sender.send(m);
    }

    fn get_response_history(
        &self,
        partner: NodeId,
        history: Vec<ChatMessage>,
        sender: Sender<Message>,
    ) {
        let m = Message::new(
            self.id,
            partner,
            MessageData::ResponseHistory {
                partner: self.id,
                history,
            },
        );
        sender.send(m);
    }

    fn send_text_message_to(&self, receiver: NodeId, message: String, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            receiver,
            MessageData::TextMessage {
                from: self.id,
                to: receiver,
                text: message,
            },
        );
        sender.send(m);
    }
}

impl Fragmenter for ClientLeonardo {
    fn assemble(mut message_fragments: Vec<Fragment>) -> Message {
        message_fragments.sort_by(|a, b| a.fragment_index.cmp(&b.fragment_index));
        let mut data = Vec::new();
        for fragment in message_fragments {
            data.extend(fragment.data);
        }
        let message: Message = bincode::deserialize(&data).unwrap();
        message
    }

    fn disassemble(msg: Message) -> VecDeque<Fragment> {
        let mut fragments_u8 = msg.into_u8();

        //reversing so popping gets the first element
        fragments_u8.reverse();

        let mut send_fragments: VecDeque<Fragment> = VecDeque::new();

        //frag_number is the number of fragments needed to send the message
        let frag_number = (fragments_u8.len() as f64 / FRAGMENT_DSIZE as f64).ceil() as u64;

        for i in 0..frag_number {
            let mut len: u8 = 0;
            let mut data: [u8; FRAGMENT_DSIZE] = [0; FRAGMENT_DSIZE];

            for i in 0..FRAGMENT_DSIZE {
                if let Some(byte) = fragments_u8.pop() {
                    data[i] = byte;
                    len += 1;
                } else {
                    break;
                }
            }

            send_fragments.push_back(Fragment {
                fragment_index: i,
                total_n_fragments: frag_number,
                length: len,
                data,
            });
        }

        return send_fragments;
    }
}
