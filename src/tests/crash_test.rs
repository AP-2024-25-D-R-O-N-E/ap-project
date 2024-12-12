use std::{thread::sleep, time::Duration};

use colored::Colorize;
use wg_2024::{
    controller::DroneCommand,
    network::SourceRoutingHeader,
    packet::{Ack, Fragment, Packet, PacketType},
};

use crate::initializer::network_initializer::NetworkInitializer;

#[test]
fn main() {
    super::initialize();
    let mut network_initializer =
        NetworkInitializer::new("src/topology_configs/config.toml".to_string());

    network_initializer.init_network();

    // normal send test
    let ack = Ack { fragment_index: 0 };

    let ack_packet = Packet {
        pack_type: PacketType::Ack(ack),
        routing_header: SourceRoutingHeader {
            hops: vec![0, 1, 2, 5, 3],
            hop_index: 1,
        },
        session_id: 0,
    };

    let msg = Fragment {
        fragment_index: 0,
        total_n_fragments: 1,

        length: 1,
        data: [1; 128],
    };

    let msg_packet = Packet {
        pack_type: PacketType::MsgFragment(msg),
        routing_header: SourceRoutingHeader {
            hops: vec![0, 1, 2, 5, 3],
            hop_index: 1,
        },
        session_id: 1,
    };

    //normal behaviour

    log::info!("{} starting now", "NORMAL TEST".yellow());

    // let _ = network_initializer.get_send_channel(1).send(ack_packet.clone());
    let _ = network_initializer
        .get_send_channel(1)
        .send(msg_packet.clone());

    sleep(Duration::from_secs(2));

    //crashbehaviour test

    log::info!("{} starting now", "CRASH BEHAVIOUR".yellow());

    let _ = network_initializer
        .get_drone_command_channel(2)
        .send(DroneCommand::Crash);

    // let _ = network_initializer.get_send_channel(1).send(ack_packet.clone());
    let _ = network_initializer
        .get_send_channel(1)
        .send(msg_packet.clone());

    sleep(Duration::from_secs(2));

    // test after the crashed drone stopped existing

    log::info!("{} starting now", "DROPPED THREAD".yellow());

    let _ = network_initializer
        .get_drone_command_channel(1)
        .send(DroneCommand::RemoveSender(2));
    let _ = network_initializer
        .get_drone_command_channel(5)
        .send(DroneCommand::RemoveSender(2));

    // let _ = network_initializer.get_send_channel(1).send(ack_packet.clone());
    let _ = network_initializer
        .get_send_channel(1)
        .send(msg_packet.clone());

    sleep(Duration::from_secs(2));
}
