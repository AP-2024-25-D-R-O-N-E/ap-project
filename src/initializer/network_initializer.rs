use colored::Colorize;
use std::{
    collections::HashMap,
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
    client, server,
    simulation_controller::{
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
    handles: HashMap<NodeId, JoinHandle<()>>,
}

impl NetworkInitializer {
    pub fn new(config_path: String) -> NetworkInitializer {
        NetworkInitializer {
            config: parse_config(config_path),
            packet_channels: HashMap::new(), // packets
            node_event_channels: HashMap::new(),
            drone_command_channels: HashMap::new(),
            client_event_channels: HashMap::new(),
            client_command_channels: HashMap::new(),
            server_event_channels: HashMap::new(),
            server_command_channels: HashMap::new(),
            handles: HashMap::new(),
        }
    }

    pub fn init_network(&mut self) -> Result<SimulationController, String> {
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
        for drone in self.config.drone.iter() {
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


            let barrier_clone = Arc::clone(&drone_barrier);
            self.handles.insert(
                drone_id,
                thread::spawn(move || {
                    let mut drone = MyDrone::new(
                        drone_id,
                        command_send,
                        command_receiver,
                        packet_recv,
                        packet_send,
                        pdr,
                    );

                    log::info!(
                        "{}, {:?}",
                        format!("Initialized drone {}", drone_id).purple(),
                        drone,
                    );
                    barrier_clone.wait();
                    // run function is where the logic of the drone runs.
                    drone.run();
                }),
            );
        }
        drone_barrier.wait();
        log::info!("{}", "Drones initialized successfully!".bold().green());

        // client initialization

        let client_barrier = Arc::new(Barrier::new(self.config.client.len() + 1));
        for client in self.config.client.iter() {
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
                    let mut client = client::Client::new(
                        client_id,
                        command_send,
                        command_receiver,
                        packet_recv,
                        packet_send,
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
        for server in self.config.server.iter() {
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
                    let mut server = server::Server::new(
                        server_id,
                        command_send,
                        command_receiver,
                        packet_recv,
                        packet_send,
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
}
