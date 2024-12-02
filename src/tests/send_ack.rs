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

    network_initializer.init_network();
    sleep(Duration::from_secs(4));

    //* just some testing */
    let ack = Ack { fragment_index: 0 };

    let packet = Packet {
        pack_type: PacketType::Ack(ack),
        routing_header: SourceRoutingHeader {
            hops: vec![6, 0, 2, 4, 5, 7],
            hop_index: 1,
        },
        session_id: 0,
    };

    let _ = network_initializer.get_send_channel(0).send(packet);
    sleep(Duration::from_secs(4));
}
