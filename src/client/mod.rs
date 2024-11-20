use crate::{edge_node::EdgeNode, fragmentation::Fragmenter};

struct Client;

impl Fragmenter for Client {
    fn disassemble(
        msg: wg_2024::packet::Message,
    ) -> std::collections::HashMap<u64, wg_2024::packet::Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> wg_2024::packet::Message {
        todo!()
    }
}

impl EdgeNode for Client {
    fn new(options: crate::edge_node::EdgeNodeOptions) -> Self {
        todo!()
    }

    fn run() {
        todo!()
    }
}
