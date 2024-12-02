use colored::Colorize;
use std::{
    collections::HashMap,
    thread::{self, JoinHandle},
};

use crossbeam::channel::{unbounded, Receiver, Sender};
use wg_2024::{
    controller::{DroneCommand, NodeEvent},
    drone::{Drone, DroneOptions},
    network::NodeId,
    packet::Packet,
};

use d_r_o_n_e_drone::MyDrone;

use crate::{
    client,
    simulation_controller::structs::{ClientCommand, ClientEvent, ServerCommand, ServerEvent},
};

use super::config_parsing::{parse_config, InitConfig};

pub struct NetworkInitializer {
    config: InitConfig,
    packet_channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)>,
    node_event_channels: HashMap<NodeId, (Sender<NodeEvent>, Receiver<NodeEvent>)>,
    drone_command_channels: HashMap<NodeId, (Sender<DroneCommand>, Receiver<DroneCommand>)>,
    client_event_channels: HashMap<NodeId, (Sender<ClientEvent>, Receiver<ClientEvent>)>,
    client_command_channels: HashMap<NodeId, (Sender<ClientCommand>, Receiver<ClientCommand>)>,
    server_event_channels: HashMap<NodeId, (Sender<ServerEvent>, Receiver<ServerEvent>)>,
    server_command_channels: HashMap<NodeId, (Sender<ServerCommand>, Receiver<ServerCommand>)>,
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

    pub fn init_network(&mut self) {
        //create 3 different version since we might want the simulation controller channels to depend on node type
        for drone in self.config.drone.iter() {
            //create unbounded channel for drones
            self.packet_channels
                .insert(drone.id as u8, unbounded::<Packet>());
        }

        for client in self.config.client.iter() {
            self.packet_channels
                .insert(client.id, unbounded::<Packet>());
            //change this is a similar way to drones
            self.client_event_channels
                .insert(client.id, unbounded::<ClientEvent>());
            self.client_command_channels
                .insert(client.id, unbounded::<ClientCommand>());
        }

        for server in self.config.server.iter() {
            self.packet_channels
                .insert(server.id, unbounded::<Packet>());
            //change this is a similar way to drones
            self.server_event_channels
                .insert(server.id, unbounded::<ServerEvent>());
            self.server_command_channels
                .insert(server.id, unbounded::<ServerCommand>());
        }

        // thread creations

        for drone in self.config.drone.iter() {
            //create the channels used for simulation controller communication
            let node_event_send = unbounded::<NodeEvent>();
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
            let drone_id: NodeId = drone.id.try_into().unwrap();

            let pdr = drone.pdr as f32;

            self.handles.insert(
                drone_id,
                thread::spawn(move || {
                    let options = DroneOptions {
                        id: drone_id,
                        controller_send: command_send,
                        controller_recv: command_receiver,
                        packet_recv,
                        packet_send,
                        pdr,
                    };
                    let mut drone = MyDrone::new(options);

                    log::info!(
                        "{}, {:?}",
                        format!("Initialized drone {}", drone_id).bold().purple(),
                        drone,
                    );
                    // run function is where the logic of the drone runs.
                    drone.run();
                }),
            );
        }

        // TODO create client threads

        for client in self.config.client.iter() {}

        // TODO create server threads
        for server in self.config.server.iter() {}

        // create simulation controller and give all the join handles to it + the channels
    }

    //just for testing purposes
    pub fn get_send_channel(&self, drone_id: NodeId) -> &Sender<Packet> {
        &self.packet_channels.get(&drone_id).unwrap().0
    }
}
