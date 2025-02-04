pub mod message;

use message::Message;

use std::collections::VecDeque;

use wg_2024::packet::Fragment;

pub trait Fragmenter {
    fn disassemble(msg: Message) -> VecDeque<Fragment>;
    fn assemble(fragments: Vec<Fragment>) -> Message;
}
