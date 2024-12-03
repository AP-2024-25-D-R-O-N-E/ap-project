// #![allow(unused, dead_code)]
// use 'use' for easier naming

use initializer::network_initializer::NetworkInitializer;
use simple_logger::SimpleLogger;
use simulation_controller::visualizer::visualizer::run_gui;
use std::env;

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

    let mut simulation_controller =
        NetworkInitializer::new("src/config.toml".to_string()).init_network();
    match simulation_controller {
        Ok(sc) => run_gui("Simulation controller".to_string(), sc),
        Err(err) => log::error!("Error in network initialization: {}", err),
    }
}
