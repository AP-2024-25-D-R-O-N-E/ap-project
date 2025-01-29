use std::{thread::sleep, time::Duration};

use crate::{initializer::network_initializer::NetworkInitializer, simulation_controller::ServerCommand};
use simple_logger::SimpleLogger;
use wg_2024::{network::SourceRoutingHeader, packet::*};

#[test]
fn send_flood_request() {
    super::initialize();
    let mut network_initializer =
        NetworkInitializer::new("src/topology_configs/config.toml".to_string());

    let sc = network_initializer.init_network().unwrap();
    sleep(Duration::from_millis(100));

    sc.send_control_packet(ServerCommand::NetworkInitialized, 3);
    sleep(Duration::from_secs(4));
}
