use std::{collections::HashMap, thread};

use crossbeam::channel::{unbounded, Receiver, Sender};
use wg_2024::{controller::{DroneCommand, NodeEvent}, drone::{Drone, DroneOptions}, network::NodeId, packet::Packet};

use d_r_o_n_e_drone::MyDrone;

use super::config_parsing::{parse_config, InitConfig};

pub struct NetworkInitializer {
    config: InitConfig,
    packet_channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)>,
    node_even_channels: HashMap<NodeId, (Sender<NodeEvent>, Receiver<NodeEvent>)>,
    drone_command_channels: HashMap<NodeId, (Sender<DroneCommand>, Receiver<DroneCommand>)>,
}

impl NetworkInitializer {
    pub fn new(config_path: String) -> NetworkInitializer {
        NetworkInitializer {
            config: parse_config(config_path),
            packet_channels: HashMap::new(),
            node_even_channels: HashMap::new(),
            drone_command_channels: HashMap::new(),
        }
    }

    pub fn init_network(&mut self) {
        //create 3 different version since we might want the simulation controller channels to depend on node type
        for drone in self.config.drone.iter() {
            //create unbounded channel for drones
            self.packet_channels
                .insert(drone.id, unbounded::<Packet>());
            self.node_even_channels
                .insert(drone.id, unbounded::<NodeEvent>());
            self.drone_command_channels
                .insert(drone.id, unbounded::<DroneCommand>());
        }

        for drone in self.config.drone.iter() {
            //clones all the sender channels for the connected drones
            let mut sender_channels: HashMap<NodeId, Sender<Packet>> = HashMap::new();

            for connected_drone in drone.connected_node_ids.iter() {
                sender_channels.insert(
                    *connected_drone,
                    self.packet_channels
                        .get(&connected_drone)
                        .unwrap()
                        .0
                        .clone(),
                );
            }

            let packet_receiver = self
                .packet_channels
                .get(&drone.id)
                .unwrap()
                .1
                .clone();
            let command_receiver = self.drone_command_channels.get(&drone.id).unwrap().1.clone();
            let command_send = self.node_even_channels.get(&drone.id).unwrap().0.clone();

            // since the thread::spawn function will take ownership of the values, we need to copy or clone them to not have problems with the Vec
            let drone_id: NodeId = drone.id.try_into().unwrap();

            let pdr = drone.pdr as f32;

            thread::spawn(move || {

                let options = DroneOptions {
                    id: drone_id,
                    controller_send: command_send,
                    controller_recv: command_receiver,
                    packet_recv: packet_receiver,
                    packet_send: sender_channels,
                    pdr,
                };
                let mut drone = MyDrone::new(options);

                println!("my thread's drone: {:?}", drone);
                // run function is where the logic of the drone runs.
                drone.run();
            });
        }
    }

    //just for testing purposes
    pub fn get_send_channel(&self, drone_id: NodeId) -> &Sender<Packet> {
        &self.packet_channels.get(&drone_id).unwrap().0
    }
}
