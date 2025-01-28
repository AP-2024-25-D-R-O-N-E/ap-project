pub mod crash_test;
pub mod send_ack;
pub mod send_flood_request;
pub mod send_msg;
pub mod simulation;

use log::*;
use simple_logger::SimpleLogger;
use std::sync::Once;

// This makes the logger availvable for each test that requires it, while ensuring
// SimpleLogger::new() is called only once
static INIT: Once = Once::new();
pub fn initialize() {
    INIT.call_once(|| {
        SimpleLogger::new()
            .without_timestamps()
            .env()
            .init()
            .unwrap();
    });
}

pub use send_ack::*;
pub use send_flood_request::*;
pub use send_msg::*;
