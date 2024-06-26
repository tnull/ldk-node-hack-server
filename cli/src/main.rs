use clap::{Parser, Subcommand};
use client::ServerHackClient;
use protos::{
	GetBalancesRequest, GetNodeStatusRequest, ListChannelsRequest, OnchainReceiveRequest,
	OnchainSendRequest, PaymentsHistoryRequest,
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
	SendOnchain { address: String, amount_sats: Option<u64> },
	NodeBalances,
	ListChannels,
	PaymentsHistory,
}

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	let client = ServerHackClient::new(cli.base_url);

	match cli.command {
		Commands::NodeStatus => {
			match client.get_node_status(&GetNodeStatusRequest {}).await {
				Ok(response) => {
					println!("Node status: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting node status: {:?}", e);
				},
			};
		},
		Commands::NewAddress {} => {
			match client.get_new_funding_address(OnchainReceiveRequest {}).await {
				Ok(address) => {
					println!("New address: {:?}", address);
				},
				Err(e) => {
					eprintln!("Error getting new funding address: {:?}", e);
				},
			};
		},
		Commands::SendOnchain { address, amount_sats } => {
			match client.send_onchain(OnchainSendRequest { address, amount_sats }).await {
				Ok(response) => {
					println!("Sent onchain: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error sending onchain: {:?}", e);
				},
			};
		},
		Commands::NodeBalances {} => {
			match client.get_node_balances(GetBalancesRequest {}).await {
				Ok(response) => {
					println!("Node balances: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting node balances: {:?}", e);
				},
			};
		},
		Commands::ListChannels {} => {
			match client.list_channels(ListChannelsRequest {}).await {
				Ok(response) => {
					println!("Channels: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting list of channels: {:?}", e);
				},
			};
		},
		Commands::PaymentsHistory => {
			match client.get_payments_history(&PaymentsHistoryRequest {}).await {
				Ok(response) => {
					println!("Payments history: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting payments history: {:?}", e);
				},
			}
		},
	}
}
