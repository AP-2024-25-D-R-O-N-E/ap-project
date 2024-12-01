use std::{thread::sleep, time::Duration};

use crate::initializer::network_initializer::NetworkInitializer;
use wg_2024::{
    network::SourceRoutingHeader,
    packet::{Ack, Fragment, Packet, PacketType},
};

#[test]
fn send_msg() {
    let mut network_initializer = NetworkInitializer::new("src/config.toml".to_string());

    network_initializer.init_network();

    //* just some testing */
    let ack = Ack { fragment_index: 0 };

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

    let msg = Fragment {
        fragment_index: 0,
        total_n_fragments: 1,

        length: 1,
        data: [1; 80],
    };

    let packet = Packet {
        pack_type: PacketType::MsgFragment(msg),
        routing_header: SourceRoutingHeader {
            hops: vec![0, 1, 2],
            hop_index: 0,
        },
        session_id: 1,
    };

    let _ = network_initializer.get_send_channel(0).send(packet);
    sleep(Duration::from_secs(4));
}
