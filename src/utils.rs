use std::path::Path;

use crate::config::{Config, JsonConfig};
use anyhow::Result;

pub fn read_config_from_json<P: AsRef<Path>>(config_path: P) -> Result<Config> {
	let contents = std::fs::read_to_string(config_path.as_ref())?;
	let json_config: JsonConfig = serde_json::from_str(&contents)?;
	Ok(Config::from(json_config))
}

#[cfg(test)]
mod tests {
	use std::{net::SocketAddr, str::FromStr};

	use ldk_node::{bitcoin::Network, lightning::ln::msgs::SocketAddress, LogLevel};

	use super::*;

	#[test]
	fn test_read_json_config_from_file() {
		let storage_path = std::env::temp_dir();
		let config_file_name = "config.json";

		let json_config = r#"{
            "esplora_server_url": "localhost:3000",
            "listening_addr": "localhost:3001",
            "log_level": "info",
            "network": "regtest",
            "rest_service_addr": "127.0.0.1:3002"
        }"#;

		std::fs::write(storage_path.join(config_file_name), json_config).unwrap();

		assert_eq!(
			read_config_from_json(storage_path.join(config_file_name)).unwrap(),
			Config {
				esplora_server_url: "localhost:3000".to_string(),
				listening_addr: SocketAddress::from_str("localhost:3001").unwrap(),
				log_level: LogLevel::Info,
				network: Network::Regtest,
				rest_service_addr: SocketAddr::from_str("127.0.0.1:3002").unwrap(),
			}
		)
	}
}
