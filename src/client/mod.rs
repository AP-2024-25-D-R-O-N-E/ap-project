use std::collections::HashMap;

use crossbeam::{
    channel::{Receiver, Sender},
    select,
};
use wg_2024::{controller::Command, network::NodeId, packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, Packet, PacketType}};

use crate::{edge_node::EdgeNode, fragmentation::Fragmenter};


#[derive(Debug)]
pub struct Client {
    pub id: NodeId,
    pub scs: Sender<Command>,
    pub scr: Receiver<Command>,
    pub pr: Receiver<Packet>,
    pub ps: HashMap<NodeId, Sender<Packet>>
}

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
        Client { 
            id: options.id,
            scs: options.sim_contr_send,
            scr: options.sim_contr_recv,
            pr: options.packet_recv,
            ps: options.packet_send 
        }
    }

    
    fn run(&self) {
        loop {
            select! {
                recv(self.pr) -> packet_res => {
                    if let Ok(packet) = packet_res {

                        //remember to remove the underscores when you actually start using the variable ig
                        match &packet.pack_type {
                            PacketType::Nack(nack)=>self.manage_nack(nack),
                            PacketType::Ack(ack)=>self.manage_ack(ack),
                            PacketType::MsgFragment(fragment)=>self.manage_msg_fragment(fragment),
                            //  ...these two are jet to be defined...
                            PacketType::FloodRequest(flood_request) => self.manage_flood_request(flood_request), 
                            PacketType::FloodResponse(flood_response) => self.manage_flood_response(flood_response),
                        }

                        
                    }
                },
                recv(self.scr) -> command_res => {
                    if let Ok(command) = command_res {
                        //here goes the handling fo the sim controller
                    }
                }
            }
        }
    }
}

impl Client {
    // these are all sample function to handle messages

    fn manage_nack(&self, nack: &Nack){
        //resend the packet
        println!("client {} received a nack: {:?}", self.id, nack);
    }

    fn manage_ack(&self, ack: &Ack){
        //free memory of message vector
        println!("client {} received an ack: {:?}", self.id, ack);
    }

    fn manage_msg_fragment(&self, msg: &Fragment){
        //call to the assembler
        println!("client {} received an fragment: {:?}", self.id, msg);
    }

    fn manage_flood_request(&self, fr: &FloodRequest){
        //call to the assembler
        println!("client {} received a flood request: {:?}", self.id, fr);
    }

    fn manage_flood_response(&self, fr: &FloodResponse){
        //call to the assembler
        println!("client {} received a flood response: {:?}", self.id, fr);
    }


}
