use std::{path::Path, str::FromStr};

use anyhow::{anyhow, Result};
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::{bitcoin::Network, LogLevel};
use serde::{Deserialize, Serialize};

const CONFIG_FILE_NAME: &str = "config.json";

#[derive(PartialEq, Eq, Debug)]
pub struct Config {
	pub esplora_server_url: String,
	pub listening_addr: SocketAddress,
	pub log_level: LogLevel,
	pub network: Network,
}

impl Config {
	pub fn new<P: AsRef<Path>>(storage_dir_path: P) -> Result<Self> {
        let contents = std::fs::read_to_string(storage_dir_path.as_ref().join(CONFIG_FILE_NAME))?;
		let json_config: JsonConfig = serde_json::from_str(&contents)?;
		let listening_addr = SocketAddress::from_str(&json_config.listening_addr)
			.map_err(|e| anyhow!("Invalid listening_addr '{}': {e}", json_config.listening_addr))?;
		let log_level = match json_config.log_level.to_lowercase().as_str() {
			"gossip" => LogLevel::Gossip,
			"trace" => LogLevel::Trace,
			"debug" => LogLevel::Debug,
			"info" => LogLevel::Info,
			"warn" => LogLevel::Warn,
			"error" => LogLevel::Error,
			_ => return Err(anyhow!(
				"Unsupported log level: {}. Use one of [gossip, trace, debug, info, warn, error] ",
				json_config.log_level
			)),
		};
		let config = Config {
			esplora_server_url: json_config.esplora_server_url,
			listening_addr,
			log_level,
			network: json_config.network,
		};
		Ok(config)
	}
}

#[derive(Deserialize, Serialize)]
struct JsonConfig {
	esplora_server_url: String,
	listening_addr: String,
	log_level: String,
	network: Network,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_config_from_file() {
        let storage_path = std::env::temp_dir();

        let json_config = r#"{
            "esplora_server_url": "localhost:3000",
            "listening_addr": "localhost:3001",
            "log_level": "info",
            "network": "regtest"
        }"#;

        std::fs::write(storage_path.join(CONFIG_FILE_NAME), json_config).unwrap();

        assert_eq!(
            Config::new(storage_path).unwrap(),
            Config {
                esplora_server_url: "localhost:3000".to_string(),
                listening_addr: SocketAddress::from_str("localhost:3001").unwrap(),
                log_level: LogLevel::Info,
                network: Network::Regtest,
            }
        );
	}

}
