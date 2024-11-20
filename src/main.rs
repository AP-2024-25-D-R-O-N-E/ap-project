//import modules
mod initializer;

use std::{
    collections::HashMap,
    fs,
    thread::{self, sleep},
    time::{Duration, Instant},
};

// use 'use' for easier naming
use initializer::{
    config_parsing::InitConfig,
    network_initializer::{self, NetworkInitializer},
};
use wg_2024::{
    network::SourceRoutingHeader,
    packet::{Ack, Packet, PacketType},
};

struct SimulationControllerCommand {
    //temporary
}

fn main() {
    let mut network_initializer = NetworkInitializer::new("src/config.toml".to_string());

    network_initializer.init_network();

    //* just some testing */
    let ack = Ack {
        fragment_index: 0,
        time_received: Instant::now(),
    };

    let packet = Packet {
        pack_type: PacketType::Ack(ack),
        routing_header: SourceRoutingHeader {
            hops: vec![0, 1, 2],
            hop_index: 0,
        },
        session_id: 0,
    };

    let _ = network_initializer.get_send_channel(0).send(packet);
    sleep(Duration::from_secs(4));

    // was simply testing
}
