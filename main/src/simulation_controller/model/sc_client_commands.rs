use std::path::PathBuf;

use wg_2024::network::NodeId;

use super::{ClientCommand, SimulationController};

impl SimulationController {
    pub fn send_client_start_flood(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => match channel.send(ClientCommand::StartFlooding) {
                Ok(_) => log::trace!("-> send_client_start_flood"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn send_get_peers(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => match channel.send(ClientCommand::GetResponseClient) {
                Ok(_) => log::trace!("-> send_get_peers"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn register(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => match channel.send(ClientCommand::RegisterAsClient) {
                Ok(_) => log::trace!("-> register"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn unregister(&self, node_id: NodeId) {
        match self.client_command_channels.get(&node_id) {
            Some(channel) => match channel.send(ClientCommand::UnregisterAsClient) {
                Ok(_) => log::trace!("-> unregister"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn send_txt_msg(&self, node_from: NodeId, node_to: NodeId, msg: String) {
        match self.client_command_channels.get(&node_from) {
            Some(channel) => match channel.send(ClientCommand::SendTextMessageTo {
                receiver: node_to,
                message: msg,
            }) {
                Ok(_) => log::trace!("-> send_txt_msg"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn send_file_msg(&self, node_from: NodeId, node_to: NodeId, file_path: PathBuf) {
        match self.client_command_channels.get(&node_from) {
            Some(channel) => match channel.send(ClientCommand::SendFileMessageTo {
                receiver: node_to,
                file_path,
            }) {
                Ok(_) => log::trace!("-> send_file_msg"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }

    pub fn open_chat_with(&self, node_from: NodeId, node_to: NodeId) {
        match self.client_command_channels.get(&node_from) {
            Some(channel) => match channel.send(ClientCommand::OpenChatWith(node_to)) {
                Ok(_) => log::trace!("-> open_chat_with"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }
}
