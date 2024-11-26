use std::{collections::HashMap, hash::Hash, thread};

use crossbeam::channel::{unbounded, Receiver, Sender};
use wg_2024::{controller::Command, drone::Drone, network::NodeId, packet::Packet};
use crate::edge_node::EdgeNodeOptions;
use d_r_o_n_e_drone::MyDrone;
use crate::client::Client as MyClient;
use crate::edge_node::EdgeNode;

use super::config_parsing::{parse_config, InitConfig};

pub struct NetworkInitializer {
    config: InitConfig,
    packet_channels: HashMap<NodeId, (Sender<Packet>, Receiver<Packet>)>,
    sc_channels: HashMap<NodeId, (Sender<Command>, Receiver<Command>)>,
}

impl NetworkInitializer {
    pub fn new(config_path: String) -> NetworkInitializer {
        NetworkInitializer {
            config: parse_config(config_path),
            packet_channels: HashMap::new(),
            sc_channels: HashMap::new(),
        }
    }

    pub fn init_network(&mut self) {
        //create 3 different version since we might want the simulation controller channels to depend on node type
        for drone in self.config.drone.iter() {
            //create unbounded channel for drones
            self.packet_channels
                .insert(drone.id as u8, unbounded::<Packet>());
            self.sc_channels
                .insert(drone.id as u8, unbounded::<Command>());
        }

        for drone in self.config.drone.iter() {
            //clones all the sender channels for the connected drones
            let mut sender_channels: HashMap<NodeId, Sender<Packet>> = HashMap::new();

            for connected_drone in drone.connected_node_ids.iter() {
                sender_channels.insert(
                    *connected_drone as u8,
                    self.packet_channels
                        .get(&(*connected_drone as u8))
                        .unwrap()
                        .0
                        .clone(),
                );
            }

            let packet_receiver = self
                .packet_channels
                .get(&(drone.id as u8))
                .unwrap()
                .1
                .clone();
            let command_receiver = self.sc_channels.get(&(drone.id as u8)).unwrap().1.clone();
            let command_send = self.sc_channels.get(&(drone.id as u8)).unwrap().0.clone();

            // since the thread::spawn function will take ownership of the values, we need to copy or clone them to not have problems with the Vec
            let drone_id: NodeId = drone.id.try_into().unwrap();

            let pdr = drone.pdr as f32;

            thread::spawn(move || {
                //template for droneoptions, do not uncomment since it will take ownership of the values

                // let _option = DroneOptions {
                //     id: drone_id,
                //     sim_contr_send: command_send,
                //     sim_contr_recv: command_receiver,
                //     packet_recv: packet_receiver,
                //     pdr,
                // };

                //this call will only be useful until the protocol for drone is fixed

                let mut drone = MyDrone::new_drone(
                    drone_id,
                    command_send,
                    command_receiver,
                    sender_channels,
                    packet_receiver,
                    pdr,
                );

                println!("my thread's drone: {:?}", drone);
                // run function is where the logic of the drone runs.
                drone.run();

                




            });
        }

        //now putting all the clients in the channel hashmaps
        for client in self.config.client.iter() {
            self.packet_channels.insert(client.id as u8, unbounded::<Packet>());
            self.sc_channels.insert(client.id as u8, unbounded::<Command>());
        }

        //like for drones we need to clone everything we pass to the thread, that will take ownership
        for client in self.config.client.iter(){
            //cloning the channels
            let mut senders: HashMap<NodeId, Sender<Packet>> = HashMap::new();

            //first cloning the senders
            for c in client.connected_drone_ids.iter(){
                senders.insert(
                    *c, 
                    self
                        .packet_channels
                        .get(c)
                        .unwrap()
                        .0
                        .clone()
                );
            }

            let receiver = self
                                            .packet_channels
                                            .get(&client.id)
                                            .unwrap()
                                            .1
                                            .clone();

            let sim_command_receiver = self
                                                                            .sc_channels
                                                                            .get(&client.id)
                                                                            .unwrap()
                                                                            .1
                                                                            .clone();

            let sim_command_sender = self
                                                                        .sc_channels
                                                                        .get(&client.id)
                                                                        .unwrap()
                                                                        .0
                                                                        .clone();
            let client_id = client.id;


            thread::spawn(move || {
                let client = MyClient::new(
                    EdgeNodeOptions{
                                id: client_id,
                                sim_contr_send: sim_command_sender,
                                sim_contr_recv: sim_command_receiver,
                                packet_recv: receiver,
                                packet_send: senders
                            }
                );

                println!("new thread client: {:?}", client);

                client.run();


            });






        }


    }

    //just for testing purposes
    pub fn get_send_channel(&self, drone_id: NodeId) -> &Sender<Packet> {
        &self.packet_channels.get(&drone_id).unwrap().0
    }
}
