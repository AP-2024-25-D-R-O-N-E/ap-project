use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, RwLock},
    thread::sleep,
    time::Duration,
};

use crate::{
    fragmentation::message::{self, Message, MessageData},
    initializer::network_initializer::NetworkInitializer,
    simulation_controller::{ClientCommand, ClientEvent, ServerCommand, SimulationController},
};
use colored::Colorize;
use simple_logger::SimpleLogger;
use tempfile::{tempdir, TempDir};
use wg_2024::{network::SourceRoutingHeader, packet::*};

#[test]
fn client_test() {
    super::initialize();

    let temp_dir: Arc<TempDir> = Arc::new(tempdir().unwrap());

    let mut network_initializer = NetworkInitializer::new(
        "src/topology_configs/config_no_pdr.toml".to_string(),
        temp_dir.clone(),
    );

    let sc = network_initializer.init_network().unwrap();
    sleep(Duration::from_millis(100));

    sc.send_control_packet(ServerCommand::NetworkInitialized, 3);

    sc.client_command_channels[&0].send(ClientCommand::StartFlooding);

    sleep(Duration::from_millis(500));

    sc.client_command_channels[&0].send(ClientCommand::RegisterAsClient);

    sleep(Duration::from_secs(4));

    //sc.client_command_channels[&0].send(ClientCommand::UnregisterAsClient);

    //sleep(Duration::from_secs(4));

    //register_client2(&sc);

    sc.client_command_channels[&0].send(ClientCommand::OpenChatWith(7));

    sleep(Duration::from_secs(4));

    sc.client_command_channels[&0].send(ClientCommand::SendTextMessageTo {
        receiver: 7,
        message: "hello".to_string(),
    });

    sleep(Duration::from_secs(4));
}

/*
fn register_client2(sc: &SimulationController) {
    let message = Message::new(7, 3, MessageData::RegisterAsClient(7));

    let fragments = crate::tests::client_test::disassemble(message);

    for fragment in fragments.iter() {
        let packet = Packet {
            pack_type: PacketType::MsgFragment(fragment.clone()),
            routing_header: SourceRoutingHeader {
                hops: vec![7, 6, 4, 2, 3],
                hop_index: 1,
            },
            session_id: 0,
        };

        sc.send_msg_fragment(packet);
    }
}

fn assemble(mut fragments: Vec<Fragment>) -> Message {
    // sort fragments by index before assembling
    fragments.sort_by(|a, b| a.fragment_index.cmp(&b.fragment_index));

    let mut message_data: Vec<u8> = Vec::new();
    for fragment in fragments {
        if fragment.length < 128 {
            message_data.extend(&fragment.data[0..fragment.length as usize]);
        } else {
            message_data.extend(&fragment.data);
        }
    }

    Message::from_u8(message_data)
}

fn disassemble(msg: Message) -> std::collections::VecDeque<Fragment> {
    let mut message_data = msg.into_u8();

    message_data.reverse();

    let mut fragments: VecDeque<Fragment> = VecDeque::new();
    let frag_numbers = (message_data.len() as f64 / 128.0).ceil() as u64;

    for i in 0..frag_numbers {
        let mut fragment_data: [u8; 128] = [0; 128];
        let mut lenght: u8 = 0;
        for index in 0..128 {
            if let Some(byte) = message_data.pop() {
                fragment_data[index] = byte;
                lenght += 1;
            } else {
                break;
            }
        }

        fragments.push_back(Fragment {
            fragment_index: i as u64,
            total_n_fragments: frag_numbers,
            length: lenght,
            data: fragment_data,
        });
    }
    fragments
}
*/
