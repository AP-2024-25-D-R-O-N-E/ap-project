use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Condvar, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use colored::Colorize;
use crossbeam::channel::{select_biased, unbounded, Receiver, Sender};
use egui_graphs::{Edge, Node};
use petgraph::{
    prelude::{GraphMap, StableGraph},
    Undirected,
};
use wg_2024::{
    network::{NodeId, SourceRoutingHeader},
    packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, NodeType, Packet, PacketType},
};

use crate::{
    fragmentation::{message::Message, Fragmenter},
    simulation_controller::structs::{ClientCommand, ClientEvent},
};

use super::ClientTrait;

pub struct Client{
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

impl ClientTrait for Client {
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
            Client::sender_thread(
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
            Client::command_handler_thread(id);
        }));

        self.receiver_thread(ready_s, nack_s);

    }
}

impl Fragmenter for Client {
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

impl Client{
    fn receiver_thread(&mut self, ready_send: Sender<(NodeId, u64)>, nack_send: Sender<Packet>){
        loop {
            select_biased!(
                recv(self.scr) -> cmd => {
                    if let Ok(command) = cmd {
                        match command {
                            ClientCommand::NetworkInitialized => self.initiate_flood(),
                            ClientCommand::AddSender(id, sender) => self.add_sender(id, sender),
                            ClientCommand::RemoveSender(id) => self.remove_sender(id),
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


}

impl Client {
    fn manage_nack(&self, nack: &Nack) {
        //resend the packet
        log::debug!(
            "{} {} received a nack: {:?}",
            "↳ client".green(),
            self.id,
            nack
        );
    }

    fn manage_ack(&self, ack: &Ack) {
        //free memory of message vector
        log::debug!(
            "{} {} received an ack: {:?}",
            "↳ client".green(),
            self.id,
            ack
        );
    }

    fn manage_msg_fragment(&self, msg: &Fragment) {
        //call to the assembler
        log::debug!(
            "{} {} received a fragment: {:?}",
            "↳ client".green(),
            self.id,
            msg
        );
    }

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

    fn manage_flood_response(&mut self, fr: &FloodResponse) {
        //call to the assembler
        log::debug!(
            "{} {} received a flood response: {:?}",
            "↳ client".green(),
            self.id,
            fr
        );

        // check that the flood response is ours
        if fr.path_trace[0].0 == self.id {
            let node = match fr.path_trace[0].1 {
                NodeType::Drone => (fr.path_trace[0].0, HashableNodeType::Drone),
                _ => (fr.path_trace[0].0, HashableNodeType::Edge),
            };

            let mut current_index = self.topology.add_node(node);

            for value in fr.path_trace.iter().skip(1) {
                let node = match value.1 {
                    NodeType::Drone => (value.0, HashableNodeType::Drone),
                    _ => (value.0, HashableNodeType::Edge),
                };
                let new_node = self.topology.add_node(node);
                self.topology.add_edge(current_index, new_node, ());
                current_index = new_node;
            }
        }

        log::info!("{} {:?}", "Client 1 topology: ".green(), self.topology);
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
            self.scs.send(ClientEvent::PacketSent(packet));
        }
    }

    fn add_sender(&mut self, id: NodeId, sender: Sender<Packet>) {
        self.ps.insert(id, sender);
    }

    fn remove_channel(&mut self, id: NodeId) {
        self.ps.remove(&id);
    }
}
