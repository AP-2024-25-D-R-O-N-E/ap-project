use std::collections::HashMap;

use wg_2024::packet::{Fragment, Message};

pub trait Fragmenter {
    fn disassemble(msg: Message) -> HashMap<u64, Fragment>;
    fn assemble(fragments: Vec<Fragment>) -> Message;
}
