use wg_2024::packet::Packet;

/// From client to controller
#[derive(Debug, Clone)]
pub enum ClientEvent {
    PacketSent(Packet),
    PacketDropped(Packet),
}

/// From controller to client
#[derive(Debug, Clone)]
pub enum ClientCommand {

}

/// From server to controller
#[derive(Debug, Clone)]
pub enum ServerEvent {
    PacketSent(Packet),
    PacketDropped(Packet),
}

/// From controller to server
#[derive(Debug, Clone)]
pub enum ServerCommand {

}

//for now I'll use these, maybe in the future we can make them polymorphic of some sort or just remove 
//the edge_node struct and make them separate

/// From edge node to controller
#[derive(Debug, Clone)]
pub enum EdgeNodeEvent {
    PacketSent(Packet),
    PacketDropped(Packet),
}

/// From controller to edge node
#[derive(Debug, Clone)]
pub enum EdgeNodeCommand {

}