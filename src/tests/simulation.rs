use crate::initializer::network_initializer::NetworkInitializer;

#[test]
fn simulation() {
    super::initialize();
    let mut network_initializer = NetworkInitializer::new("src/config.toml".to_string());

    let simulation_controller = network_initializer.init_network().unwrap();

}
