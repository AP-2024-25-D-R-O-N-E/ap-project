use std::{collections::VecDeque, sync::Arc, thread::sleep, time::Duration};

use crate::{
    fragmentation::message::{Message, MessageData},
    initializer::network_initializer::NetworkInitializer,
    simulation_controller::{ServerCommand, SimulationController},
};
use colored::Colorize;
use tempfile::{tempdir, TempDir};
use wg_2024::{network::SourceRoutingHeader, packet::*};

#[test]
fn server_functionality() {
    super::initialize();
    let temp_dir: Arc<TempDir> = Arc::new(tempdir().unwrap());
    let network_initializer = NetworkInitializer::new(
        "src/topology_configs/config_no_pdr.toml".to_string(),
        temp_dir,
    );

    let sc = network_initializer.init_network().unwrap();
    sleep(Duration::from_millis(100));

    sc.send_control_packet(ServerCommand::NetworkInitialized, 3);

    sleep(Duration::from_millis(100));

    log::debug!("{}", "Sending register client".red());

    register_client1(&sc);
    register_client2(&sc);
    sleep(Duration::from_millis(100));

    log::debug!("{}", "Request clients".red());
    request_clients(&sc);
    sleep(Duration::from_millis(100));

    log::debug!("{}", "Sending text messages".red());
    // text_message(&sc, "Hello Worldafdlsjkfdjaskfjla;kfjdlkfds;jklafljk;fljk;fad;jklasdfjkl;dfal;kfjpoaiwejfopiahfeoiahfpoiehwafpoihepoifajspodfijaepoifhaopiehfpaosidjfaopsidfpoaijfeopiawhfpoaewuhfoieawhfpoiahweofhawefou".to_string());
    text_message(&sc, "text 2".to_string());
    text_message(&sc, "why not lol".to_string());
    sleep(Duration::from_millis(100));

    log::debug!("{}", "Requesting chat history".red());
    chat_history(&sc);
    sleep(Duration::from_millis(100));

    log::debug!("{}", "Sending text messages".red());
    text_message(&sc, "text 3".to_string());
    text_message(&sc, "text 4".to_string());
    sleep(Duration::from_millis(100));

    log::debug!("{}", "Unregistering client".red());
    unregister_client1(&sc);
    sleep(Duration::from_millis(100));

    log::debug!("{}", "Request clients".red());
    request_clients(&sc);
    sleep(Duration::from_millis(100));

    // sc.send_control_packet(ServerCommand::NetworkInitialized, 3);
    sleep(Duration::from_secs(4));
}

fn register_client1(sc: &SimulationController) {
    let message = Message::new(0, 3, MessageData::RegisterAsClient(0));

    let fragments = disassemble(message);

    for fragment in fragments.iter() {
        let packet = Packet {
            pack_type: PacketType::MsgFragment(fragment.clone()),
            routing_header: SourceRoutingHeader {
                hops: vec![0, 1, 2, 3],
                hop_index: 1,
            },
            session_id: 0,
        };

        sc.send_msg_fragment(packet);
    }
}

fn register_client2(sc: &SimulationController) {
    let message = Message::new(7, 3, MessageData::RegisterAsClient(7));

    let fragments = disassemble(message);

    for fragment in fragments.iter() {
        let packet = Packet {
            pack_type: PacketType::MsgFragment(fragment.clone()),
            routing_header: SourceRoutingHeader {
                hops: vec![7, 6, 4, 2, 3],
                hop_index: 2,
            },
            session_id: 0,
        };

        sc.send_msg_fragment(packet);
    }
}

fn unregister_client1(sc: &SimulationController) {
    let message = Message::new(0, 3, MessageData::UnregisterAsClient(0));

    let fragments = disassemble(message);

    for fragment in fragments.iter() {
        let packet = Packet {
            pack_type: PacketType::MsgFragment(fragment.clone()),
            routing_header: SourceRoutingHeader {
                hops: vec![0, 1, 2, 3],
                hop_index: 1,
            },
            session_id: 0,
        };

        sc.send_msg_fragment(packet);
    }
}

fn request_clients(sc: &SimulationController) {
    let message = Message::new(0, 3, MessageData::RequestClients(0));

    let fragments = disassemble(message);

    for fragment in fragments.iter() {
        let packet = Packet {
            pack_type: PacketType::MsgFragment(fragment.clone()),
            routing_header: SourceRoutingHeader {
                hops: vec![0, 1, 2, 3],
                hop_index: 1,
            },
            session_id: 0,
        };

        sc.send_msg_fragment(packet);
    }
}

fn text_message(sc: &SimulationController, text: String) {
    let message = Message::new(
        0,
        3,
        MessageData::TextMessage {
            from: 0,
            to: 7,
            text,
        },
    );

    let fragments = disassemble(message);

    for fragment in fragments.iter() {
        let packet = Packet {
            pack_type: PacketType::MsgFragment(fragment.clone()),
            routing_header: SourceRoutingHeader {
                hops: vec![0, 1, 2, 3],
                hop_index: 1,
            },
            session_id: 0,
        };

        sc.send_msg_fragment(packet);
    }
}

fn chat_history(sc: &SimulationController) {
    let message = Message::new(
        0,
        3,
        MessageData::RequestHistory {
            requester: 0,
            partner: 7,
        },
    );

    let fragments = disassemble(message);

    for fragment in fragments.iter() {
        let packet = Packet {
            pack_type: PacketType::MsgFragment(fragment.clone()),
            routing_header: SourceRoutingHeader {
                hops: vec![0, 1, 2, 3],
                hop_index: 1,
            },
            session_id: 0,
        };

        sc.send_msg_fragment(packet);
    }
}

fn disassemble(msg: Message) -> std::collections::VecDeque<Fragment> {
    let mut message_data = msg.as_u8();

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
            fragment_index: i,
            total_n_fragments: frag_numbers,
            length: lenght,
            data: fragment_data,
        });
    }
    fragments
}
