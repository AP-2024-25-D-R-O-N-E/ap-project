use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use crate::initializer::network_initializer::NetworkInitializer;
use wg_2024::{
    network::SourceRoutingHeader,
    packet::{Ack, Fragment, Packet, PacketType},
};

#[test]
fn main() {
    let mut network_initializer =
        NetworkInitializer::new("src/topology_configs/config.toml".to_string());

    let sc = network_initializer.init_network().unwrap();
    sleep(Duration::from_secs(100));

    let ack = Ack { fragment_index: 0 };

    let packet = Packet {
        pack_type: PacketType::Ack(ack),
        routing_header: SourceRoutingHeader {
            hops: vec![0, 1, 2, 3],
            hop_index: 1,
        },
        session_id: 0,
    };

    sc.send_ack(packet);
    sleep(Duration::from_secs(100));
}
