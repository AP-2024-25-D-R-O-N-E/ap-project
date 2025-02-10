use std::{
    collections::{HashMap, HashSet, VecDeque},
    ffi::OsString,
    path::PathBuf,
    sync::{Arc, Condvar, Mutex, RwLock},
    thread::{self},
};

use colored::Colorize;
use crossbeam::channel::{select_biased, unbounded, Receiver, Sender};
use petgraph::{algo, data::Build, prelude::GraphMap, Undirected};
use tempfile::TempDir;
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{
        self, Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType,
        FRAGMENT_DSIZE,
    },
};

use crate::{
    fragmentation::{
        file_handling::{byte_vec_to_file, file_to_byte_vec, raw_vec_to_chat_vec},
        message::{Message, MessageData, RawChatMessage},
        Fragmenter,
    },
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

use super::{
    utils::{ClientChannels, LockRef, SenderThreadChannels},
    ClientTrait,
};

pub struct ClientLeonardo {
    id: NodeId,
    flood_id: u64,
    chat_server_id: Arc<Mutex<NodeId>>,

    //channels with sim controller
    sim_contr_send: Sender<ClientEvent>,
    sim_contr_recv: Receiver<ClientCommand>,
    //channels with drones
    packet_recv: Receiver<Packet>,
    packet_send: LockRef<HashMap<NodeId, Sender<Packet>>>,

    //topology of the net, weight depends on prd
    topology: LockRef<GraphMap<NodeId, f64, Undirected>>,

    //buffers for the fragments
    fragment_buffer: LockRef<HashMap<(NodeId, u64), Vec<Fragment>>>,
    ack_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>,
    topology_modified: Arc<Mutex<bool>>,
    edge_nodes: LockRef<HashSet<NodeId>>,
    condv: Arc<Condvar>,
    temp_dir: Arc<TempDir>,
}

impl ClientTrait for ClientLeonardo {
    fn new(
        id: NodeId,
        sim_contr_send: Sender<ClientEvent>,
        sim_contr_recv: Receiver<ClientCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        temp_dir: Arc<TempDir>,
    ) -> Self
    where
        Self: Sized,
    {
        ClientLeonardo {
            id,
            flood_id: 0,
            chat_server_id: Arc::new(Mutex::new(0)),
            sim_contr_send,
            sim_contr_recv,
            packet_recv,
            packet_send: Arc::new(RwLock::new(packet_send)),
            topology: Arc::new(RwLock::new(GraphMap::new())),
            fragment_buffer: Arc::new(RwLock::new(HashMap::new())),
            ack_buffer: Arc::new(Mutex::new(HashMap::new())),
            topology_modified: Arc::new(Mutex::new(false)),
            edge_nodes: Arc::new(RwLock::new(HashSet::new())),
            condv: Arc::new(Condvar::new()),
            temp_dir,
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
        let ack_buffer = self.ack_buffer.clone();
        let topology = self.topology.clone();
        let edge_nodes = self.edge_nodes.clone();
        let condv = self.condv.clone();

        thread::spawn(move || {
            Self::sender_thread(
                id,
                SenderThreadChannels::new(
                    packet_send,
                    sim_contr_send,
                    nacks_recv,
                    fragment_receiver,
                ),
                condv,
                ack_buffer,
                topology,
                edge_nodes,
            );
        });

        let fragment_buffer = self.fragment_buffer.clone();
        let ready_for_handler = ready_for_handler.clone();
        let thread_receiver = thread_receiver.clone();
        let sim_control_send = self.sim_contr_send.clone();
        let fragment_sender = fragment_sender.clone();
        let chat_server_id = self.chat_server_id.clone();
        let temp_dir = self.temp_dir.clone();

        thread::spawn(move || {
            Self::message_handler_thread(
                chat_server_id,
                fragment_buffer,
                ready_for_handler,
                temp_dir,
                ClientChannels::new(thread_receiver, fragment_sender, sim_control_send),
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
                                self.get_response_history(partner, thread_sender.clone());
                            },
                            ClientCommand::SendTextMessageTo{receiver, message} => {
                                self.send_text_message_to(receiver, message, thread_sender.clone());
                            },
                            ClientCommand::SendFileMessageTo{receiver, file_path } => {
                                self.send_file_message_to(receiver, file_path, thread_sender.clone());
                            },
                        }
                    }
                },
                recv(self.packet_recv) -> packet => {

                    if let Ok(p) = packet {
                        if let Ok(()) = self.sim_contr_send.send(ClientEvent::PacketReceived(p.clone())) { log::debug!("{} Packet received: {}", "↳ client".purple(), p) } else { log::error!("{} Packet received but couldn't be sent to the simulation controller", "↳ client".purple()) }

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
                        }
                    }
                }
            );
        }
    }

    fn sender_thread(
        id: NodeId,

        sender_thread_channels: SenderThreadChannels,

        condv: Arc<Condvar>,
        ack_packet_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>,
        topology: LockRef<GraphMap<NodeId, f64, Undirected>>,
        edge_nodes: LockRef<HashSet<NodeId>>,
    ) {
        // temporary number
        const MAX_OUTPUT_BUFFER: usize = 1024;
        let SenderThreadChannels {
            packet_sender,
            sim_contr_send,
            nack_recv,
            fragment_recv,
        } = sender_thread_channels;

        loop {
            select_biased!(
                recv(nack_recv) -> nack_res => {
                    if let Ok(packet) = nack_res {
                        // recalculate route if topology was modified

                        let _ = *packet.routing_header.hops.last().unwrap();

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

                        let packet = Packet {
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
        server_id: Arc<Mutex<NodeId>>,
        fragment_buffers: LockRef<HashMap<(NodeId, u64), Vec<Fragment>>>,
        ready: Receiver<(NodeId, u64)>,
        temp_dir: Arc<TempDir>,
        client_channels: ClientChannels,
    ) {
        let mut session_id = 1;

        let ClientChannels {
            command_recv,
            fragment_sender,
            sim_send,
        } = client_channels;

        loop {
            select_biased!(
                recv(command_recv) -> frag_res => {
                    //disassembly of the message and send to sender thread

                    let fragments = Self::disassemble(frag_res.unwrap());

                    let chat_server_id = *server_id.lock().unwrap();

                    for fragment in fragments {
                        if let Ok(()) = fragment_sender.send((chat_server_id, session_id, fragment)) { log::debug!("{} Message sent: {}", "↳ client".purple(), session_id) } else { log::error!("{} Message sent but couldn't be sent to the sender thread", "↳ client".purple()) }
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
                                Self::response_history(partner, history, sim_send.clone(), temp_dir.clone());
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
                            MessageData::FileMessage { from, to, file, file_name, extension } => {
                                Self::file_message_received(from, to, file, file_name, extension, sim_send.clone(), temp_dir.clone());
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
        avoid_nodes: LockRef<HashSet<NodeId>>,
        topology: LockRef<GraphMap<NodeId, f64, Undirected>>,
    ) -> Vec<NodeId> {
        let topology_lock = topology.read().unwrap();

        let path = algo::astar(
            &*topology_lock,
            start_id,
            |end| end == destination_id,
            |(a, b, _)| {
                if destination_id == b || destination_id == a {
                    return 1;
                }
                let avoid_nodes_lock = avoid_nodes.read().unwrap();
                if avoid_nodes_lock.contains(&a) || avoid_nodes_lock.contains(&b) {
                    topology_lock.edge_count()
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
        packet_sender: LockRef<HashMap<u8, Sender<Packet>>>,
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

        if res.is_err() {
            log::error!("The send inside channel gave an error, this shouldn't be happening");
        } else if let Ok(()) = sim_contr_send.send(ClientEvent::PacketSent(packet.clone())) {
            log::debug!("{} Packet sent: {}", "↳ client".purple(), packet);
        } else {
            log::error!(
                "{} Packet sent but couldn't be sent to the simulation controller",
                "↳ client".purple()
            );
        }
    }

    //------------------Receiver Thread------------------//

    fn manage_ack(&self, header_vec: Vec<NodeId>, ack: Ack, s_id: u64) {
        let mut ack_buffer_lock = self.ack_buffer.lock().unwrap();
        let key = (s_id, ack.fragment_index);

        //if receiving ack then removing it from the the ack buffer
        if ack_buffer_lock.remove(&key).is_some() {
            log::debug!(
                "{} Ack received for fragment {} of message {}",
                "↳ client".purple(),
                ack.fragment_index,
                s_id
            );
            self.condv.notify_all();
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
            let curr_weight = *topology_lock.edge_weight(h1, h2).unwrap();
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
                    let curr_weight = *topology_lock.edge_weight(h1, h2).unwrap();
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
            if let Ok(()) = resend.send(p.clone()) {
                log::debug!("{} Packet resent: {}", "↳ client".purple(), p);
            } else {
                log::error!("{} Packet resent but couldn't be sent", "↳ client".purple());
            }
            self.condv.notify_all();
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

        for (id, node_type) in flood_res.path_trace.iter().skip(1) {
            let next = topology_lock.add_node(*id);
            match node_type {
                NodeType::Client => {
                    if id != &self.id {
                        edge_nodes_lock.insert(*id);
                    }
                }
                NodeType::Server => {
                    edge_nodes_lock.insert(*id);
                    let mut server_id_lock = self.chat_server_id.lock().unwrap();
                    *server_id_lock = *id;
                }
                _ => {}
            }

            //inizializza tutti i weight a 1
            topology_lock.add_edge(index, next, 0.0);
            index = next;
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
            if let std::collections::hash_map::Entry::Vacant(e) =
                fragment_buffer_lock.entry((p_source, id))
            {
                e.insert(vec![f]);
                if tot == 1 {
                    if let Ok(()) = ready.send((p_source, id)) {
                        log::debug!(
                            "{} Message ready to be assembled ready {}",
                            "↳ client".purple(),
                            id
                        );
                    } else {
                        log::error!(
                            "{} Ready message sent but couldn't be sent",
                            "↳ client".purple()
                        );
                    }
                }
            } else {
                let buffer = fragment_buffer_lock.get_mut(&(p_source, id)).unwrap();
                buffer.push(f);

                if buffer.len() == tot as usize {
                    if let Ok(()) = ready.send((p_source, id)) {
                        log::debug!(
                            "{} Message ready to be assembled ready {}",
                            "↳ client".purple(),
                            id
                        );
                    } else {
                        log::error!(
                            "{} Ready message sent but couldn't be sent",
                            "↳ client".purple()
                        );
                    }
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

        if let Err(_packet) = res {
            log::error!("The send inside channel gave an error, this shouldn't be happening");
        } else if let Ok(()) = self
            .sim_contr_send
            .send(ClientEvent::PacketSent(packet.clone()))
        {
            log::debug!("{} Packet sent: {}", "↳ client".purple(), packet);
        } else {
            log::error!(
                "{} Packet sent but couldn't be sent to the simulation controller",
                "↳ client".purple()
            );
        }
    }

    //-----------------Controller Thread-----------------//

    //events to simulation controller

    fn send_response(res: Vec<u8>, sim_send: Sender<ClientEvent>) {
        if let Ok(()) = sim_send.send(ClientEvent::ResponseClientsReceived(res)) {
            log::debug!("{} Peers fetched", "↳ client".purple());
        } else {
            log::error!(
                "{} Couldn't send ResponseClientsReceived",
                "↳ client".purple()
            );
        }
    }

    fn client_ack(sim_send: Sender<ClientEvent>) {
        if let Ok(()) = sim_send.send(ClientEvent::AcknolewdgedAsClient) {
            log::debug!("{} Acknowledged as client", "↳ client".purple());
        } else {
            log::error!("{} Couldn't send AcknolewdgedAsClient", "↳ client".purple());
        }
    }

    fn response_history(
        partner: NodeId,
        history: Vec<RawChatMessage>,
        sim_send: Sender<ClientEvent>,
        temp_dir: Arc<TempDir>,
    ) {
        if let Ok(()) = sim_send.send(ClientEvent::ResponseHistoryReceived {
            partner,
            history: raw_vec_to_chat_vec(history, temp_dir.clone()),
        }) {
            log::debug!("{} History fetched", "↳ client".purple());
        } else {
            log::error!(
                "{} Couldn't send ResponseHistoryReceived",
                "↳ client".purple()
            );
        }
    }

    fn unregistered_sender_error(sim_send: Sender<ClientEvent>) {
        if let Ok(()) = sim_send.send(ClientEvent::UnregisteredSenderError) {
            log::debug!("{} Unregistered sender error", "↳ client".purple());
        } else {
            log::error!(
                "{} Couldn't send UnregisteredSenderError",
                "↳ client".purple()
            );
        }
    }

    fn unregistered_recipient_error(sim_send: Sender<ClientEvent>) {
        if let Ok(()) = sim_send.send(ClientEvent::UnregisteredRecipientError) {
            log::debug!("{} Unregistered recipient error", "↳ client".purple());
        } else {
            log::error!(
                "{} Couldn't send UnregisteredRecipientError",
                "↳ client".purple()
            );
        }
    }

    fn unsupported_message_type_error(sim_send: Sender<ClientEvent>) {
        if let Ok(()) = sim_send.send(ClientEvent::UnsupportedMessageTypeError) {
            log::debug!("{} Unsupported message type error", "↳ client".purple());
        } else {
            log::error!(
                "{} Couldn't send UnsupportedMessageTypeError",
                "↳ client".purple()
            );
        }
    }

    fn text_message_received(
        from: NodeId,
        to: NodeId,
        text: String,
        sim_send: Sender<ClientEvent>,
    ) {
        if let Ok(()) = sim_send.send(ClientEvent::TextMessage { from, to, text }) {
            log::debug!("{} Text message received", "↳ client".purple());
        } else {
            log::error!("{} Couldn't send TextMessage", "↳ client".purple());
        }
    }

    fn file_message_received(
        from: NodeId,
        to: NodeId,
        file: Vec<u8>,
        file_name: OsString,
        extension: OsString,
        sim_send: Sender<ClientEvent>,
        temp_dir: Arc<TempDir>,
    ) {
        if let Ok(()) = sim_send.send(ClientEvent::FileMessage {
            from,
            to,
            file_path: byte_vec_to_file(file_name, extension, file, temp_dir.clone()).unwrap(),
        }) {
            log::debug!("{} File message received", "↳ client".purple());
        } else {
            log::error!("{} Couldn't send FileMessage", "↳ client".purple());
        }
    }

    //commands from simulation controller

    fn initiate_flood(&mut self) {
        for (_, recv) in self.packet_send.read().unwrap().iter() {
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

            if res.is_err() {
                log::error!("The send inside channel gave an error, this shouldn't be happening");
            } else if let Ok(()) = self
                .sim_contr_send
                .send(ClientEvent::PacketSent(packet.clone()))
            {
                log::debug!("{} Packet sent: {}", "↳ client".purple(), packet);
            } else {
                log::error!(
                    "{} Packet sent but couldn't be sent to the simulation controller",
                    "↳ client".purple()
                );
            }
        }
    }

    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        println!("{} {}", " -> added sender ".green(), id);
        self.packet_send.write().unwrap().insert(id, sender);
    }

    fn remove_sender(&mut self, id: NodeId) {
        self.packet_send.write().unwrap().remove(&id);
    }

    fn get_response_clients(&self, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            *self.chat_server_id.lock().unwrap(),
            MessageData::RequestClients(self.id),
        );
        if let Ok(()) = sender.send(m) {
            log::debug!("{} Requesting clients", "↳ client".purple());
        } else {
            log::error!("{} Couldn't send RequestClients", "↳ client".purple());
        }
    }

    fn register_as_client(&self, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            *self.chat_server_id.lock().unwrap(),
            MessageData::RegisterAsClient(self.id),
        );
        if let Ok(()) = sender.send(m) {
            log::debug!("{} Registering as client", "↳ client".purple());
        } else {
            log::error!("{} Couldn't send RegisterAsClient", "↳ client".purple());
        }
    }

    fn unregister_as_client(&self, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            *self.chat_server_id.lock().unwrap(),
            MessageData::UnregisterAsClient(self.id),
        );
        if let Ok(()) = sender.send(m) {
            log::debug!("{} Unregistering as client", "↳ client".purple());
        } else {
            log::error!("{} Couldn't send UnregisterAsClient", "↳ client".purple());
        }
    }

    fn get_response_history(&self, partner: NodeId, sender: Sender<Message>) {
        let m = Message::new(
            self.id,
            partner,
            MessageData::RequestHistory {
                requester: self.id,
                partner,
            },
        );
        match sender.send(m) {
            Ok(()) => log::debug!("{} Requesting history", "↳ client".purple()),
            Err(err) => log::error!(
                "{} Couldn't send RequestHistory {}",
                "↳ client".purple(),
                err
            ),
        }
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
        match sender.send(m) {
            Ok(()) => log::debug!("{} Sending text message", "↳ client".purple()),
            Err(err) => log::error!("{} Couldn't send TextMessage {}", "↳ client".purple(), err),
        }
    }

    fn send_file_message_to(&self, receiver: NodeId, file_path: PathBuf, sender: Sender<Message>) {
        let (file, file_name, extension) = file_to_byte_vec(file_path).unwrap();

        // create file locally only if you're not the receiver
        if receiver != self.id {
            let file_path = byte_vec_to_file(
                file_name.clone(),
                extension.clone(),
                file.clone(),
                self.temp_dir.clone(),
            )
            .unwrap();

            let local_file = ClientEvent::CreatedFileLocal {
                from: self.id,
                to: receiver,
                file_path,
            };

            match self.sim_contr_send.send(local_file) {
                Ok(()) => log::debug!("{} File created locally", "↳ client".purple()),
                Err(err) => log::error!(
                    "{} Couldn't send CreatedFileLocal {}",
                    "↳ client".purple(),
                    err
                ),
            }
        }

        let m = Message::new(
            self.id,
            receiver,
            MessageData::FileMessage {
                from: self.id,
                to: receiver,
                file,
                file_name,
                extension,
            },
        );
        match sender.send(m) {
            Ok(()) => log::debug!("{} Sending file message", "↳ client".purple()),
            Err(err) => log::error!("{} Couldn't send FileMessage {}", "↳ client".purple(), err),
        }
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
        let mut fragments_u8 = msg.as_u8();

        //reversing so popping gets the first element
        fragments_u8.reverse();

        let mut send_fragments: VecDeque<Fragment> = VecDeque::new();

        //frag_number is the number of fragments needed to send the message
        let frag_number = (fragments_u8.len() as f64 / FRAGMENT_DSIZE as f64).ceil() as u64;

        for i in 0..frag_number {
            let mut len: u8 = 0;
            let mut data: [u8; FRAGMENT_DSIZE] = [0; FRAGMENT_DSIZE];

            for item in data.iter_mut().take(FRAGMENT_DSIZE) {
                if let Some(byte) = fragments_u8.pop() {
                    *item = byte;
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

        send_fragments
    }
}
