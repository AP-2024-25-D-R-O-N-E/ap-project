use std::fs;

pub use wg_2024::config::Config as InitConfig;

pub fn parse_config(config_path: String) -> InitConfig {
    let config_data = fs::read_to_string(config_path).expect("Unable to read config file");

    toml::from_str(&config_data).expect("Unable to parse TOML")
}
