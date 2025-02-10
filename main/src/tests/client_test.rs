use std::{sync::Arc, thread::sleep, time::Duration};

use tempfile::{tempdir, TempDir};

use crate::{
    initializer::network_initializer::NetworkInitializer,
    simulation_controller::{ClientCommand, ServerCommand},
};

#[test]
fn client_test() {
    super::initialize();

    let temp_dir: Arc<TempDir> = Arc::new(tempdir().unwrap());

    let network_initializer = NetworkInitializer::new(
        "src/topology_configs/config_no_pdr.toml".to_string(),
        temp_dir,
    );

    let sc = network_initializer.init_network().unwrap();
    sleep(Duration::from_millis(100));

    sc.send_control_packet(ServerCommand::NetworkInitialized, 3);

    match sc.client_command_channels[&0].send(ClientCommand::StartFlooding) {
        Ok(()) => log::trace!("-> send_client_start_flood to client 0"),
        Err(err) => log::error!("Channel error {}", err),
    }

    sleep(Duration::from_millis(500));

    match sc.client_command_channels[&0].send(ClientCommand::RegisterAsClient) {
        Ok(()) => log::trace!("-> register client 0"),
        Err(err) => log::error!("Channel error {}", err),
    }

    sleep(Duration::from_secs(4));

    match sc.client_command_channels[&0].send(ClientCommand::OpenChatWith(7)) {
        Ok(()) => log::trace!("-> open client 0 chat with client 7"),
        Err(err) => log::error!("Channel error {}", err),
    }

    sleep(Duration::from_secs(4));

    match sc.client_command_channels[&0].send(ClientCommand::SendTextMessageTo {
        receiver: 7,
        message: "hello".to_string(),
    }) {
        Ok(()) => log::trace!("-> send message from client 0 to client 7"),
        Err(err) => log::error!("Channel error {}", err),
    }

    sleep(Duration::from_secs(4));
}
