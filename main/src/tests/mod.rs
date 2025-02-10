pub mod assembler_test;
pub mod client_test;
pub mod crash_test;

pub mod message_serialization_test;
pub mod server_features;

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
