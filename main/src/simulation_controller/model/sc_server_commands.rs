use wg_2024::network::NodeId;

use super::{ServerCommand, SimulationController};

impl SimulationController {
    pub fn send_server_start_flood(&self, node_id: NodeId) {
        if let Some(channel) = self.server_command_channels.get(&node_id) {
            match channel.send(ServerCommand::NetworkInitialized) {
                Ok(()) => log::trace!("-> send_server_start_flood"),
                Err(err) => log::error!("Channel error {}", err),
            }
        } else {
            log::error!("Specified client does not exist")
        }
    }
}
