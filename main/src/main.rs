// #![allow(unused, dead_code)]
// use 'use' for easier naming

use initializer::network_initializer::NetworkInitializer;
use simple_logger::SimpleLogger;
use simulation_controller::visualizer::ui::run_gui;
use std::{
    fs,
    sync::{Arc, Mutex, RwLock},
};
use tempfile::{tempdir, TempDir};

mod client;
mod fragmentation;
mod initializer;
mod server;
mod simulation_controller;

#[cfg(test)]
mod tests;

fn main() {
    // just in case its not possible to do everything from the simulation
    // let args: Vec<String> = env::args().collect();
    // if args.len() < 2 {
    //     log::error!("Usage: {}", "cargo run -- simulation");
    // }
    SimpleLogger::new()
        .without_timestamps()
        .env()
        .init()
        .unwrap();

    // temp dir for the entire simulation, behaves like a static variable
    let temp_dir: Arc<TempDir> = Arc::new(tempdir().unwrap());

    let simulation_controller = NetworkInitializer::new(
        "main/src/topology_configs/config.toml".to_string(),
        temp_dir.clone(),
    )
    .init_network();
    match simulation_controller {
        Ok(sc) => run_gui("Simulation controller".to_string(), sc),
        Err(err) => log::error!("Error in network initialization: {}", err),
    }

    // temp_dir drop isn't properly cleaning up the directory so we do it manually
    fs::remove_dir_all(temp_dir.path()).unwrap();
}
