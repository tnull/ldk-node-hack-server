use clap::{Parser, Subcommand};
use client::ServerHackClient;
use protos::{
	GetBalancesRequest, GetNodeStatusRequest, GetNodeStatusRequest, OnchainReceiveRequest,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	#[arg(short, long, default_value = "localhost:3000")]
	base_url: String,

	#[command(subcommand)]
	command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
	NodeStatus,
	NewAddress,
	NodeBalances,
}

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	let client = ServerHackClient::new(cli.base_url);

	match cli.command {
		Commands::NodeStatus {} => {
			match client.get_node_status(GetNodeStatusRequest {}).await {
				Ok(response) => {
					println!("Node status: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting node status: {:?}", e);
				},
			};
		},
		Commands::NewAddress => {
			match client.get_new_funding_address(OnchainReceiveRequest {}).await {
				Ok(address) => {
					println!("New address: {:?}", address);
				},
				Err(e) => {
					eprintln!("Error getting new funding address: {:?}", e);
				},
			}
		},
		Commands::NodeBalances => {
			match client.get_node_balances(GetBalancesRequest {}).await {
				Ok(response) => {
					println!("Node balances: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting node balances: {:?}", e);
				},
			};
		},
	}
}
