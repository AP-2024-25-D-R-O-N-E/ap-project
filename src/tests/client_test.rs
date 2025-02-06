use std::{collections::VecDeque, thread::sleep, time::Duration};

use crate::{
    fragmentation::message::{self, Message, MessageData},
    initializer::network_initializer::NetworkInitializer,
    simulation_controller::{ClientCommand, ClientEvent, ServerCommand, SimulationController},
};
use colored::Colorize;
use simple_logger::SimpleLogger;
use wg_2024::{network::SourceRoutingHeader, packet::*};

#[test]
fn client_test() {
    super::initialize();
    let mut network_initializer =
        NetworkInitializer::new("src/topology_configs/config_no_pdr.toml".to_string());

    let sc = network_initializer.init_network().unwrap();
    sleep(Duration::from_millis(100));

    sc.send_control_packet(ServerCommand::NetworkInitialized, 3);

    sc.client_command_channels[&0].send(ClientCommand::StartFlooding);

    sleep(Duration::from_millis(500));

    sc.client_command_channels[&0].send(ClientCommand::RegisterAsClient);

    sleep(Duration::from_secs(4));

}