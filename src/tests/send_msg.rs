use std::{thread::sleep, time::Duration};

use crate::initializer::network_initializer::NetworkInitializer;
use wg_2024::{
    network::SourceRoutingHeader,
    packet::{Ack, Fragment, Packet, PacketType},
};

#[test]
pub fn send_msg() {
    super::initialize();
    let mut network_initializer =
        NetworkInitializer::new("src/topology_configs/config.toml".to_string());

    network_initializer.init_network();

    let msg = Fragment {
        fragment_index: 0,
        total_n_fragments: 1,

        length: 1,
        data: [1; 80],
    };

    let packet = Packet {
        pack_type: PacketType::MsgFragment(msg),
        routing_header: SourceRoutingHeader {
            hops: vec![7, 6, 4, 2, 1, 0],
            hop_index: 1,
        },
        session_id: 1,
    };

    let _ = network_initializer.get_send_channel(6).send(packet);
    sleep(Duration::from_secs(4));
}
