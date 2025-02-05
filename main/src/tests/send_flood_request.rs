use std::{thread::sleep, time::Duration};

use crate::initializer::network_initializer::NetworkInitializer;
use simple_logger::SimpleLogger;
use wg_2024::{network::SourceRoutingHeader, packet::*};

#[test]
fn send_flood_request() {
    super::initialize();
    let mut network_initializer =
        NetworkInitializer::new("src/topology_configs/config.toml".to_string());

    let sc = network_initializer.init_network().unwrap();
    sleep(Duration::from_millis(100));

    let flood_req = FloodRequest {
        path_trace: vec![(0, NodeType::Client)],
        flood_id: 0,
        initiator_id: 0,
    };

    let packet = Packet {
        pack_type: PacketType::FloodRequest(flood_req),
        routing_header: SourceRoutingHeader {
            hops: vec![0, 1],
            hop_index: 1,
        },
        session_id: 0, // it'll be whatever for now
    };

    sc.send_flood_request(packet, 1);
    sleep(Duration::from_secs(4));
}
