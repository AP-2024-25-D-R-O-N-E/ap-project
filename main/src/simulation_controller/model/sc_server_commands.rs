use wg_2024::network::NodeId;

use super::{ServerCommand, SimulationController};

impl SimulationController {
    pub fn send_server_start_flood(&self, node_id: NodeId) {
        match self.server_command_channels.get(&node_id) {
            Some(channel) => match channel.send(ServerCommand::NetworkInitialized) {
                Ok(_) => log::trace!("-> send_server_start_flood"),
                Err(err) => log::error!("Channel error {}", err),
            },
            None => log::error!("Specified client does not exist"),
        }
    }
}
