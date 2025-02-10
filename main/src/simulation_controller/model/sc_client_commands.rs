use std::path::PathBuf;

use wg_2024::network::NodeId;

use super::{ClientCommand, SimulationController};

impl SimulationController {
    pub fn send_client_start_flood(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(ClientCommand::StartFlooding);
            }
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn send_get_peers(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(ClientCommand::GetResponseClient);
            }
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn register(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(ClientCommand::RegisterAsClient);
            }
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn unregister(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(ClientCommand::UnregisterAsClient);
            }
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn send_txt_msg(&self, node_from: NodeId, node_to: NodeId, msg: String) {
        match self.client_command_channels.get(&node_from) {
            Some(channel) => {
                channel.send(ClientCommand::SendTextMessageTo {
                    receiver: node_to,
                    message: msg,
                });
            }
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn send_file_msg(&self, node_from: NodeId, node_to: NodeId, file_path: PathBuf) {
        match self.client_command_channels.get(&node_from) {
            Some(channel) => {
                channel.send(ClientCommand::SendFileMessageTo {
                    receiver: node_to,
                    file_path,
                });
            }
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn open_chat_with(&self, node_from: NodeId, node_to: NodeId) {
        match self.client_command_channels.get(&node_from) {
            Some(channel) => {
                channel.send(ClientCommand::OpenChatWith(node_to));
            }
            None => log::error!("Specified client does not exist"),
        }
    }
}
