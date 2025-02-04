use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use colored::Colorize;
use crossbeam::channel::{select_biased, unbounded, Receiver, Sender};
use egui_graphs::{Edge, Node};
use petgraph::{algo, prelude::{GraphMap, StableGraph}, Undirected};
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{self, Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType},
};

use crate::{
    fragmentation::{message::Message, Fragmenter},
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

use super::ClientTrait;

pub struct ClientLuca {
    id: NodeId,
    scs: Sender<ClientEvent>,
    scr: Receiver<ClientCommand>,
    packet_r: Receiver<Packet>,
    packet_s: Arc<RwLock<HashMap<NodeId, Sender<Packet>>>>,
    flood_id: u64,
    topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>,
    fragment_buffer: Arc<RwLock<HashMap<(NodeId, u64), Vec<Fragment>>>>,
    ack_packet_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>,
    topology_modified: Arc<Mutex<bool>>,
    edge_nodes: Arc<RwLock<HashSet<NodeId>>>,
}

impl ClientTrait for ClientLuca {
    fn new(id: NodeId, scs: Sender<ClientEvent>, scr: Receiver<ClientCommand>, packet_r: Receiver<Packet>, packet_s: HashMap<NodeId, Sender<Packet>>) -> Self
    where
        Self: Sized,
    {
        Self {
            id,
            scs,
            scr,
            packet_r,
            packet_s: Arc::new(RwLock::new(packet_s)),
            flood_id: 0,
            topology: Arc::new(RwLock::new(GraphMap::new())),
            fragment_buffer: Arc::new(RwLock::new(HashMap::new())),
            ack_packet_buffer: Arc::new(Mutex::new(HashMap::new())),
            topology_modified: Arc::new(Mutex::new(false)),
            edge_nodes: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    fn run(&mut self) {
        let mut threads: Vec<JoinHandle<()>> = Vec::new();

        let packet_s = self.packet_s.clone();
        let scs = self.scs.clone();
        let (nack_s, nack_r) = unbounded::<Packet>();
        let (fragment_s, fragment_r) = unbounded::<(NodeId, u64, Fragment)>();
        let condv = Condvar::new();
        let ack_packet_buffer = self.ack_packet_buffer.clone();
        let topology = self.topology.clone();
        let id = self.id;
        let topology_modified = self.topology_modified.clone();
        let edge_nodes = self.edge_nodes.clone();

        threads.push(thread::spawn(move || {
            ClientLuca::sender_thread(
                id,
                packet_s,
                scs,
                fragment_r,
                nack_r,
                condv,
                ack_packet_buffer,
                topology,
                topology_modified,
                edge_nodes,
            );
        }));

        let (ready_s, ready_r) = unbounded::<(NodeId, u64)>();

        let id = self.id;

        threads.push(thread::spawn(move ||{
            ClientLuca::command_handler_thread(id);
        }));

        self.receiver_thread(ready_s, nack_s);

    }
}

impl Fragmenter for ClientLuca {
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

impl ClientLuca {
    fn receiver_thread(&mut self, ready_s: Sender<(NodeId, u64)>, nack_s: Sender<Packet>){
        loop {
            select_biased!(
                recv(self.scr) -> cmd => {
                    if let Ok(command) = cmd {
                        match command {
                            ClientCommand::NetworkInitialized => self.initiate_flood(),
                            ClientCommand::AddSender(id, sender) => self.add_sender(id, sender),
                            ClientCommand::RemoveSender(id) => self.remove_sender(id),
                            ClientCommand::RequestClients => todo!()
                            //Need to add other commands when defined
                        }
                    }
                },
                recv(self.packet_r) -> res => {
                    if let Ok(mut packet) = res {
                        match packet.pack_type {
                            PacketType::MsgFragment(_) => self.manage_msg_fragment(packet, ready_s.clone()),
                            PacketType::Ack(ack) => self.manage_ack(packet.session_id, ack),
                            PacketType::Nack(nack) => self.manage_nack(packet.session_id, nack, nack_s.clone()),
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
        packet_s: Arc<RwLock<HashMap<u8, Sender<Packet>>>>,
        scs: Sender<ClientEvent>,
        fragment_r: Receiver<(NodeId, u64, Fragment)>,
        nack_r: Receiver<Packet>,
        condv: Condvar,
        ack_packet_buffer: Arc<Mutex<HashMap<(u64, u64), Packet>>>,
        topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>,
        topology_modified: Arc<Mutex<bool>>,
        edge_nodes: Arc<RwLock<HashSet<NodeId>>>,
    ) {
        // records the routing table for the client (updated when an update to the topology is made)
        let mut routing_table: HashMap<NodeId, Vec<NodeId>> = HashMap::new();

        const MAX_OUTPUT_BUFFER: usize = 10;

        loop {
            select_biased!(
                recv(nack_r) -> nack_res => {
                    if let Ok(mut packet) = nack_res {
                        // recalculate route if topology was modified
                        let mut topology_mod_lock = topology_modified.lock().unwrap();

                        let destination = packet.routing_header.hops.last().unwrap().clone();

                        if *topology_mod_lock {
                            Self::find_route(id, destination, &mut routing_table, topology.clone(), edge_nodes.clone());
                            *topology_mod_lock = false;
                        }

                        packet.routing_header.hops = routing_table.get(&destination).unwrap().clone();

                        let mut ack_buffer = condv
                        .wait_while(ack_packet_buffer.lock().unwrap(), |buff| {
                            buff.len() + nack_r.len() >= MAX_OUTPUT_BUFFER
                        })
                        .unwrap();

                        if let PacketType::MsgFragment(fragment) = &packet.pack_type {
                            ack_buffer.insert((packet.session_id, fragment.fragment_index), packet.clone());
                        }

                        Self::send_msg_packet(id, packet_s.clone(), scs.clone(), packet);

                    }
                },
                recv(fragment_r) -> frag_res => {
                    if let Ok((destination, session_id, fragment)) = frag_res {
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
                        let mut ack_buffer = condv
                        .wait_while(ack_packet_buffer.lock().unwrap(), |buff| {
                            buff.len() + nack_r.len() >= MAX_OUTPUT_BUFFER
                        })
                        .unwrap();

                        ack_buffer.insert((session_id, fragment_index), packet.clone());

                        Self::send_msg_packet(id, packet_s.clone(), scs.clone(), packet);
                    }
                }
            );
        }
    }

    fn command_handler_thread(id: NodeId){}

}

//Thread: receiver
impl ClientLuca {
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
            "↳ client".green(),
            self.id,
            fr
        );

        // check that the flood response is ours
        if fr.path_trace[0].0 == self.id {
            let mut topology_lock = self.topology.write().unwrap();
            let mut edge_nodes_lock = self.edge_nodes.write().unwrap();
            let mut topology_mod_lock = self.topology_modified.lock().unwrap();

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
            *topology_mod_lock = true;
        }

        log::info!("{} {:?}", "Client topology: ".green(), self.topology);
    }


    fn manage_msg_fragment(&self, packet: Packet, ready_send: Sender<(NodeId, u64)>) {
        log::debug!(
            "{} {} received a message fragment: {}",
            "↳ client".green(),
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
            let mut fragment_buffer_lock = self.fragment_buffer.write().unwrap();

            let total_frags = fragment.total_n_fragments;

            if fragment_buffer_lock.contains_key(&(packet_source, packet_msg_id)) {
                let mut frag_buffer = fragment_buffer_lock
                    .get_mut(&(packet_source, packet_msg_id))
                    .unwrap();

                frag_buffer.push(fragment);

                // if the buffer is full, tell the message handler thread to start assembling the fragments
                if frag_buffer.len() == total_frags as usize {
                    ready_send.send((packet_source, packet_msg_id));
                }
            } else {
                fragment_buffer_lock.insert((packet_source, packet_msg_id), vec![fragment]);
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
            "↳ client".green(),
            self.id,
            ack
        );

        let ack_key = (session_id, ack.fragment_index);

        let mut ack_packet_buffer_lock = self.ack_packet_buffer.lock().unwrap();

        if let Some(packet) = ack_packet_buffer_lock.remove(&ack_key) {
            log::debug!(
                "{} {} removed packet from ack buffer: {:?}",
                "↳ client".green(),
                self.id,
                packet
            );
        }
    }

    fn manage_nack(&self, session_id: u64, nack: Nack, nack_send: Sender<Packet>) {
        log::debug!(
            "{} {} received a nack: {:?}",
            "↳ client".green(),
            self.id,
            nack
        );

        // return that throws the packet away in case of an error
        let mut ret = false;

        match &nack.nack_type {
            packet::NackType::ErrorInRouting(node) => {
                self.topology.write().unwrap().remove_node(*node);
                *self.topology_modified.lock().unwrap() = true;
            }
            packet::NackType::DestinationIsDrone => {
                log::error!("Error: the destination is a drone");
                ret = true;
            }
            packet::NackType::Dropped => {}
            packet::NackType::UnexpectedRecipient(_) => {
                log::error!("Error: the recipient is not the expected one");
                ret = true;
            }
        }

        let ack_key = (session_id, nack.fragment_index);

        let mut ack_packet_buffer_lock = self.ack_packet_buffer.lock().unwrap();

        if let Some(packet) = ack_packet_buffer_lock.remove(&ack_key) {
            if ret {
                return;
            }
            nack_send.send(packet);
        } else {
            log::error!("Error: the packet was not found in the ack buffer");
        }
    }

    fn send_packet(&self, packet: Packet) {
        let next_node = packet.routing_header.hops[packet.routing_header.hop_index];
        let send_channel = &self.packet_s.read().unwrap()[&next_node];

        log::debug!(
            "{} from {} - packet: {}",
            " -> packet sent ".blue(),
            self.id,
            packet
        );

        let res = send_channel.send(packet.clone());

        if let Err(mut packet) = res {
            log::error!("Error in the send inside channel");
        } else {
            self.scs.send(ClientEvent::PacketSent(packet));
        }
    }

    fn initiate_flood(&mut self) {
        for (id, sender) in self.packet_s.read().unwrap().iter() {
            let packet = Packet {
                pack_type: PacketType::FloodRequest(FloodRequest {
                    path_trace: vec![(self.id, NodeType::Client)],
                    flood_id: self.flood_id,
                    initiator_id: self.id,
                }),
                routing_header: SourceRoutingHeader {
                    hops: vec![],
                    hop_index: 0,
                },
                session_id: 0,
            };
            self.flood_id += 1;

            let res = sender.send(packet.clone());

            if let Err(mut packet) = res {
                log::error!("Error in the send inside channel");
            } else {
                self.scs.send(ClientEvent::PacketSent(packet));
            }
        }
    }

    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.packet_s.write().unwrap().insert(id, sender);
    }

    fn remove_sender(&mut self, id: NodeId) {
        self.packet_s.write().unwrap().remove(&id);
    }

}


//Thread: Sender
impl ClientLuca {
    fn find_route(
        id: NodeId,
        destination: NodeId,
        routing_table: &mut HashMap<NodeId, Vec<NodeId>>,
        topology: Arc<RwLock<GraphMap<NodeId, (), Undirected>>>,
        edge_nodes: Arc<RwLock<HashSet<NodeId>>>,
    ){
        let topology_lock = topology.read().unwrap();

        let path = algo::astar(
            &*topology_lock,
            id,
            |finish| finish == destination,
            |(a, b, _)| {
                if destination == a || destination == b {
                    return 1;
                }
                let edge_nodes_lock = edge_nodes.read().unwrap();
                if edge_nodes_lock.contains(&b) || edge_nodes_lock.contains(&a) {
                    topology_lock.edge_count()
                } else {
                    1
                }
            },
            |_| 0,
        );

        if let Some((_, route)) = path{
            routing_table.insert(destination, route);
        }else{
            log::error!("No route found to destination {}", destination);
        }
    }

    fn send_msg_packet(
        id: NodeId,
        packet_s: Arc<RwLock<HashMap<u8, Sender<Packet>>>>,
        scs: Sender<ClientEvent>,
        packet: Packet,
    ){
        let next_node = packet.routing_header.hops[packet.routing_header.hop_index];
        let send_channel = &packet_s.read().unwrap()[&next_node];

        log::debug!(
            "{} from {} - packet: {}",
            " -> packet sent ".blue(),
            id,
            packet
        );

        let r = send_channel.send(packet.clone());

        if let Err(mut packet) = r{
            log::error!("The send inside channel gave an error")
        }else{
            scs.send(ClientEvent::PacketSent(packet));
        }
    }
}


