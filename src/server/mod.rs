use crate::{edge_node::EdgeNode, fragmentation::Fragmenter};

struct Server;

impl Fragmenter for Server {
    fn disassemble(
        msg: wg_2024::packet::Message,
    ) -> std::collections::HashMap<u64, wg_2024::packet::Fragment> {
        todo!()
    }

    fn assemble(fragments: Vec<wg_2024::packet::Fragment>) -> wg_2024::packet::Message {
        todo!()
    }
}

impl EdgeNode for Server {
    fn new(options: crate::edge_node::EdgeNodeOptions) -> Self {
        todo!()
    }

    fn run() {
        todo!()
    }
}
