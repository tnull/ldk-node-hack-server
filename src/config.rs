use std::net::SocketAddr;
use std::str::FromStr;

use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::{bitcoin::Network, LogLevel};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug)]
pub struct Config {
	pub esplora_server_url: String,
	pub listening_addr: SocketAddress,
	pub log_level: LogLevel,
	pub network: Network,
	pub rest_service_addr: SocketAddr,
}

impl From<JsonConfig> for Config {
	fn from(json_config: JsonConfig) -> Self {
		let listening_addr = SocketAddress::from_str(&json_config.listening_addr).unwrap();
		let rest_service_addr = SocketAddr::from_str(&json_config.rest_service_addr).unwrap();
		let log_level = match json_config.log_level.to_lowercase().as_str() {
			"gossip" => LogLevel::Gossip,
			"trace" => LogLevel::Trace,
			"debug" => LogLevel::Debug,
			"info" => LogLevel::Info,
			"warn" => LogLevel::Warn,
			"error" => LogLevel::Error,
			_ => panic!("Unsupported log level: {}", json_config.log_level),
		};
		Config {
			esplora_server_url: json_config.esplora_server_url,
			listening_addr,
			log_level,
			network: json_config.network,
			rest_service_addr,
		}
	}
}

#[derive(Deserialize, Serialize)]
pub struct JsonConfig {
	esplora_server_url: String,
	listening_addr: String,
	log_level: String,
	network: Network,
	rest_service_addr: String,
}
