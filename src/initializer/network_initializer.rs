use ap2024_unitn_cppenjoyers_drone::CppEnjoyersDrone;
use colored::Colorize;
use getdroned::GetDroned;
use lockheedrustin_drone::LockheedRustin;
use petgraph::{
    graph::NodeIndex,
    prelude::{StableGraph, StableUnGraph},
    Undirected,
};
use rust_roveri::RustRoveri;
use rustafarian_drone::RustafarianDrone;
use rustbusters_drone::RustBustersDrone;
use rusteze_drone::RustezeDrone;
use rusty_drones::RustyDrone;

use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Barrier},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use wg_2024::{
    controller::{DroneCommand, DroneEvent},
    drone::Drone,
    network::NodeId,
    packet::Packet,
};

use d_r_o_n_e_drone::MyDrone;

use crate::{
    client::{client_test::Client, client_test_2::Client2, ClientTrait},
    server::{server_test::Server, ServerTrait},
    simulation_controller::{
        edge::UiEdgePayload,
        node::{UiClientNode, UiDroneNode, UiNodePayload, UiNodeType, UiServerNode},
        structs::{ClientCommand, ClientEvent, ServerCommand, ServerEvent},
        SimulationController,
    },
};

use super::config_parsing::{parse_config, InitConfig};

pub struct NetworkInitializer {
    config: InitConfig,
    pub packet_channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)>,
    pub node_event_channels: HashMap<NodeId, (Sender<DroneEvent>, Receiver<DroneEvent>)>,
    pub drone_command_channels: HashMap<NodeId, (Sender<DroneCommand>, Receiver<DroneCommand>)>,
    pub client_event_channels: HashMap<NodeId, (Sender<ClientEvent>, Receiver<ClientEvent>)>,
    pub client_command_channels: HashMap<NodeId, (Sender<ClientCommand>, Receiver<ClientCommand>)>,
    pub server_event_channels: HashMap<NodeId, (Sender<ServerEvent>, Receiver<ServerEvent>)>,
    pub server_command_channels: HashMap<NodeId, (Sender<ServerCommand>, Receiver<ServerCommand>)>,
    pub topology: StableGraph<UiNodePayload, UiEdgePayload, Undirected>,

    handles: HashMap<NodeId, JoinHandle<()>>,
}

impl NetworkInitializer {
    pub fn new(config_path: String) -> NetworkInitializer {
        let config = parse_config(config_path);
        NetworkInitializer {
            packet_channels: HashMap::new(), // packets
            node_event_channels: HashMap::new(),
            drone_command_channels: HashMap::new(),
            client_event_channels: HashMap::new(),
            client_command_channels: HashMap::new(),
            server_event_channels: HashMap::new(),
            server_command_channels: HashMap::new(),
            handles: HashMap::new(),
            topology: NetworkInitializer::get_topology_from_config(&config),
            config,
        }
    }

    pub fn init_network(mut self) -> Result<SimulationController, String> {
        //create 3 different version since we might want the simulation controller channels to depend on node type
        for drone in self.config.drone.iter() {
            //create unbounded channel for drones
            self.packet_channels
                .insert(drone.id as u8, unbounded::<Packet>());
        }

        for client in self.config.client.iter() {
            self.packet_channels
                .insert(client.id, unbounded::<Packet>());
        }

        for server in self.config.server.iter() {
            self.packet_channels
                .insert(server.id, unbounded::<Packet>());
        }

        sleep(Duration::from_secs(1));

        // thread creations
        let drone_barrier = Arc::new(Barrier::new(self.config.drone.len() + 1));
        for (index, drone) in self.config.drone.iter().enumerate() {
            //create the channels used for simulation controller communication
            let node_event_send = unbounded::<DroneEvent>();
            let drone_command_rec = unbounded::<DroneCommand>();

            // clone them to give them to the drone
            let command_receiver = drone_command_rec.1.clone();
            let command_send = node_event_send.0.clone();

            // insert them inside the network initializer variable to later give them to the simulation controller
            self.node_event_channels.insert(drone.id, node_event_send);
            self.drone_command_channels
                .insert(drone.id, drone_command_rec);

            // create a hashmap of all the needed packet channels
            let packet_send = drone
                .connected_node_ids
                .iter()
                .map(|id| (*id, self.packet_channels[&id].0.clone()))
                .collect();

            // clone the packet receiver channel
            let packet_recv = self.packet_channels[&drone.id].1.clone();

            // since the thread::spawn function will take ownership of the values, we need to copy or clone them to not have problems with the Vec
            let drone_id: NodeId = drone.id;

            let pdr = drone.pdr as f32;

            let barrier_clone: Arc<Barrier> = Arc::clone(&drone_barrier);
            self.handles.insert(
                drone_id,
                Self::create_drone_thread(
                    index,
                    drone_id,
                    command_send,
                    command_receiver,
                    packet_recv,
                    packet_send,
                    pdr,
                    barrier_clone,
                ),
            );
        }
        drone_barrier.wait();
        log::info!("{}", "Drones initialized successfully!".bold().green());

        // client initialization

        let client_barrier = Arc::new(Barrier::new(self.config.client.len() + 1));
        for (index, client) in self.config.client.iter().enumerate() {
            let client_event_send = unbounded::<ClientEvent>();
            let client_command_rec = unbounded::<ClientCommand>();

            let command_receiver = client_command_rec.1.clone();
            let command_send = client_event_send.0.clone();

            self.client_event_channels
                .insert(client.id, client_event_send);
            self.client_command_channels
                .insert(client.id, client_command_rec);

            let packet_send = client
                .connected_drone_ids
                .iter()
                .map(|id| (*id, self.packet_channels[&id].0.clone()))
                .collect();

            let packet_recv = self.packet_channels[&client.id].1.clone();

            let client_id: NodeId = client.id;

            let barrier_clone = Arc::clone(&client_barrier);

            self.handles.insert(
                client_id,
                thread::spawn(move || {
                    let mut client = Self::create_client(
                        index as u8,
                        command_receiver,
                        command_send,
                        packet_send,
                        packet_recv,
                        client_id,
                    );
                    log::info!(
                        "{}, {:?}",
                        format!("Initialized client {}", client_id).bold().purple(),
                        client,
                    );

                    barrier_clone.wait();

                    client.run();
                }),
            );
        }
        client_barrier.wait();
        log::info!("{}", "Clients initialized successfully!".bold().green());

        let server_barrier = Arc::new(Barrier::new(self.config.server.len() + 1));
        for (index, server) in self.config.server.iter().enumerate() {
            let server_event_send = unbounded::<ServerEvent>();
            let server_command_rec = unbounded::<ServerCommand>();

            let command_receiver = server_command_rec.1.clone();
            let command_send = server_event_send.0.clone();

            self.server_event_channels
                .insert(server.id, server_event_send);
            self.server_command_channels
                .insert(server.id, server_command_rec);

            let packet_send = server
                .connected_drone_ids
                .iter()
                .map(|id| (*id, self.packet_channels[&id].0.clone()))
                .collect();

            let packet_recv = self.packet_channels[&server.id].1.clone();

            let server_id: NodeId = server.id;

            let barrier_clone = Arc::clone(&server_barrier);
            self.handles.insert(
                server_id,
                thread::spawn(move || {
                    let mut server = Self::create_server(
                        index as u8,
                        command_receiver,
                        command_send,
                        packet_send,
                        packet_recv,
                        server_id,
                    );
                    log::info!(
                        "{}, {:?}",
                        format!("Initialized server {}", server_id).bold().purple(),
                        server,
                    );

                    barrier_clone.wait();

                    server.run();
                }),
            );
        }
        server_barrier.wait();
        log::info!("{}", "Servers initialized successfully!".bold().green());

        Ok(SimulationController::new(self))
        // create simulation controller and give all the join handles to it + the channels
    }

    //just for testing purposes
    pub fn get_send_channel(&self, drone_id: NodeId) -> &Sender<Packet> {
        &self.packet_channels.get(&drone_id).unwrap().0
    }

    pub fn get_drone_command_channel(&self, drone_id: NodeId) -> &Sender<DroneCommand> {
        &self.drone_command_channels[&drone_id].0
    }

    /// Constructs graph from config file.
    pub fn get_topology_from_config(
        config: &InitConfig,
    ) -> StableGraph<UiNodePayload, UiEdgePayload, Undirected> {
        let mut graph = StableUnGraph::<UiNodePayload, UiEdgePayload>::default();

        let mut node_map_function: HashMap<wg_2024::network::NodeId, petgraph::graph::NodeIndex> =
            HashMap::new();
        for drone in &config.drone {
            let n = graph.add_node(UiNodePayload {
                node_type: UiNodeType::Drone(UiDroneNode::new(drone.pdr)),
                vendor: "unknown".to_string(),
                wg_id: drone.id,
            });
            node_map_function.insert(drone.id, n);
        }
        for server in &config.server {
            let n = graph.add_node(UiNodePayload {
                node_type: UiNodeType::Server(UiServerNode {}),
                vendor: "unknown".to_string(),
                wg_id: server.id,
            });
            node_map_function.insert(server.id, n);
        }
        for client in &config.client {
            let n = graph.add_node(UiNodePayload {
                node_type: UiNodeType::Client(UiClientNode {}),
                vendor: "unknown".to_string(),
                wg_id: client.id,
            });
            node_map_function.insert(client.id, n);
        }

        let mut set: HashSet<(NodeIndex, NodeIndex)> = HashSet::new();
        let mut insert_if_not_duplicated = |id1, id2| {
            let node_graph_id = *node_map_function.get(&id1).unwrap();
            let node_to_graph_id = *node_map_function.get(id2).unwrap();

            if !(set.contains(&(node_graph_id, node_to_graph_id))
                || set.contains(&(node_to_graph_id, node_graph_id)))
            {
                graph.add_edge(node_graph_id, node_to_graph_id, UiEdgePayload::default());
            }
            set.insert((node_graph_id, node_to_graph_id));
        };

        for drone in &config.drone {
            for node_to in &drone.connected_node_ids {
                insert_if_not_duplicated(drone.id, node_to);
            }
        }
        for server in &config.server {
            for node_to in &server.connected_drone_ids {
                insert_if_not_duplicated(server.id, node_to);
            }
        }
        for client in &config.client {
            for node_to in &client.connected_drone_ids {
                insert_if_not_duplicated(client.id, node_to);
            }
        }

        graph
    }

    fn create_server(
        index: u8,
        command_receiver: Receiver<ServerCommand>,
        command_send: Sender<ServerEvent>,
        packet_send: HashMap<u8, Sender<Packet>>,
        packet_recv: Receiver<Packet>,
        server_id: u8,
    ) -> Box<dyn ServerTrait> {
        match index {
            _ => Box::new(Server::new(
                server_id,
                command_send,
                command_receiver,
                packet_recv,
                packet_send,
            )),
        }
    }

    fn create_client(
        index: u8,
        command_receiver: Receiver<ClientCommand>,
        command_send: Sender<ClientEvent>,
        packet_send: HashMap<u8, Sender<Packet>>,
        packet_recv: Receiver<Packet>,
        client_id: u8,
    ) -> Box<dyn ClientTrait> {
        match index {
            0 => Box::new(Client::new(
                client_id,
                command_send,
                command_receiver,
                packet_recv,
                packet_send,
            )),
            _ => Box::new(Client2::new(
                client_id,
                command_send,
                command_receiver,
                packet_recv,
                packet_send,
            )),
        }
    }

    //creates a thread with a new drone inside it
    fn create_drone_thread(
        drone_index: usize, // index determines which implementation of drone to use
        id: NodeId,
        controller_send: Sender<DroneEvent>,
        controller_recv: Receiver<DroneCommand>,
        packet_recv: Receiver<Packet>,
        packet_send: HashMap<NodeId, Sender<Packet>>,
        pdr: f32,
        barrier_clone: Arc<Barrier>,
    ) -> JoinHandle<()> {
        let final_index = drone_index % 10;
        match final_index {
            0 => spawn_drone_thread::<RustafarianDrone>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            1 => spawn_drone_thread::<LockheedRustin>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            2 => spawn_drone_thread::<RustyDrone>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            3 => spawn_drone_thread::<RustBustersDrone>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            4 => spawn_drone_thread::<CppEnjoyersDrone>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            5 => spawn_drone_thread::<RustezeDrone>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            6 => spawn_drone_thread::<GetDroned>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            7 => spawn_drone_thread::<RustRoveri>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
            _ => spawn_drone_thread::<MyDrone>(
                id,
                controller_send,
                controller_recv,
                packet_recv,
                packet_send,
                pdr,
                barrier_clone,
            ),
        }
    }
}

fn spawn_drone_thread<T: Drone>(
    id: u8,
    controller_send: Sender<DroneEvent>,
    controller_recv: Receiver<DroneCommand>,
    packet_recv: Receiver<Packet>,
    packet_send: HashMap<u8, Sender<Packet>>,
    pdr: f32,
    barrier_clone: Arc<Barrier>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut drone = T::new(
            id,
            controller_send,
            controller_recv,
            packet_recv,
            packet_send,
            pdr,
        );

        log::info!(
            "{}",
            format!("Initialized drone Rustafarian {}", id).purple()
        );
        barrier_clone.wait();
        // run function is where the logic of the drone runs.
        drone.run();
    })
}

#[test]
fn tst() {}
