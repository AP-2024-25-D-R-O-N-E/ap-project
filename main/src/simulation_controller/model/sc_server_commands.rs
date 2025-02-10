use wg_2024::network::NodeId;

use super::{ServerCommand, SimulationController};

impl SimulationController {
    pub fn send_server_start_flood(&self, node_id: NodeId) {
        match self.server_command_channels.get(&node_id) {
            Some(channel) => {
                channel.send(ServerCommand::NetworkInitialized);
            }
            None => log::error!("Specified client does not exist"),
        }
    }
}
