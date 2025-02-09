use crate::fragmentation::message::{Message, MessageData};
use bincode;
use std::collections::VecDeque;
use wg_2024::packet::{Fragment, FRAGMENT_DSIZE};

#[test]

fn disassemble() {
    let msg = Message::new(
        0,
        3,
        MessageData::TextMessage {
            from: 5,
            to: 2,
            text: "ciao come stai".to_string(),
        },
    );

    let mut fragments_u8 = msg.into_u8();

    //reversing so popping gets the first element
    fragments_u8.reverse();

    let mut send_fragments: VecDeque<Fragment> = VecDeque::new();

    //frag_number is the number of fragments needed to send the message
    let frag_number = (fragments_u8.len() as f64 / FRAGMENT_DSIZE as f64).ceil() as u64;

    for i in 0..frag_number {
        let mut len: u8 = 0;
        let mut data: [u8; FRAGMENT_DSIZE] = [0; FRAGMENT_DSIZE];

        for i in 0..FRAGMENT_DSIZE {
            if let Some(byte) = fragments_u8.pop() {
                data[i] = byte;
                len += 1;
            } else {
                break;
            }
        }

        send_fragments.push_back(Fragment {
            fragment_index: i,
            total_n_fragments: frag_number,
            length: len,
            data,
        });
    }
    println!("{:?}", send_fragments);
    let message = assemble(send_fragments.into());
    println!("{:?}", message);
}

fn assemble(mut fragments: Vec<Fragment>) -> Message {
    // sort fragments by index before assembling
    fragments.sort_by(|a, b| a.fragment_index.cmp(&b.fragment_index));

    let mut message_data: Vec<u8> = Vec::new();
    for fragment in fragments {
        message_data.extend(&fragment.data);
    }

    Message::from_u8(message_data)
}

// fn assemble(mut message_fragments: Vec<Fragment>) -> Message {
//     message_fragments.sort_by(|a, b| a.fragment_index.cmp(&b.fragment_index));
//     let mut data = Vec::new();
//     for fragment in message_fragments {
//         data.extend(fragment.data);
//     }
//     let message: Message = bincode::deserialize(&data).unwrap();
//     message
// }
