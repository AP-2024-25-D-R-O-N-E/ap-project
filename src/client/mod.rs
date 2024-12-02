use std::collections::HashMap;

use crossbeam::channel::{select_biased, Receiver, Sender};
use wg_2024::{ network::NodeId, packet::{Ack, FloodRequest, FloodResponse, Fragment, Nack, Packet, PacketType}};

use crate::{
    fragmentation::Fragmenter,
    simulation_controller::structs::{ClientCommand, ClientEvent}
};


#[derive(Debug)]
pub struct Client {
    pub id: NodeId,
    pub scs: Sender<ClientEvent>,
    pub scr: Receiver<ClientCommand>,
    pub pr: Receiver<Packet>,
    pub ps: HashMap<NodeId, Sender<Packet>>
}

pub struct ClientOptions {
    pub id: NodeId,
    pub sim_contr_send: Sender<ClientEvent>,
    pub sim_contr_recv: Receiver<ClientCommand>,
    pub packet_recv: Receiver<Packet>,
    pub packet_send: HashMap<NodeId, Sender<Packet>>,
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

impl Client {
    pub fn new(options: ClientOptions) -> Self {
        Client { 
            id: options.id,
            scs: options.sim_contr_send,
            scr: options.sim_contr_recv,
            pr: options.packet_recv,
            ps: options.packet_send 
        }
    }

    
    pub fn run(&self) {
        loop {
            select_biased! {
                recv(self.scr) -> command_res => {
                    if let Ok(command) = command_res {
                        //here goes the handling fo the sim controller
                    }
                }

                recv(self.pr) -> packet_res => {
                    println!{"receiving something.."};
                    match packet_res {
                        //remember to remove the underscores when you actually start using the variable ig
                        Ok(packet) => { match &packet.pack_type {
                            PacketType::Nack(nack)=>self.manage_nack(nack),
                            PacketType::Ack(ack)=>self.manage_ack(ack),
                            PacketType::MsgFragment(fragment)=>self.manage_msg_fragment(fragment),
                            //  ...these two are jet to be defined...
                            PacketType::FloodRequest(flood_request) => self.manage_flood_request(flood_request), 
                            PacketType::FloodResponse(flood_response) => self.manage_flood_response(flood_response),
                        }
                        },
                        Err(error) => println!("{}", error),
                    }
                
                },
                
            }
        }
    }

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
