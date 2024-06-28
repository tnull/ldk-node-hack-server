use clap::{Parser, Subcommand};
use client::ServerHackClient;
use protos::{
	Bolt11ReceiveRequest, Bolt11SendRequest, CloseChannelRequest, ForceCloseChannelRequest,
	GetBalancesRequest, GetNodeIdRequest, GetNodeStatusRequest, GetPaymentDetailsRequest,
	ListChannelsRequest, OnchainReceiveRequest, OnchainSendRequest, OpenChannelRequest,
	PaymentsHistoryRequest,
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
	NodeId,
	NodeStatus,
	NewAddress,
	SendOnchain {
		address: String,
		amount_sats: Option<u64>,
	},
	Bolt11Receive {
		description: String,
		expiry_secs: u32,
		amount_msat: Option<u64>,
	},
	Bolt11Send {
		invoice: String,
		amount_msat: Option<u64>,
	},
	NodeBalances,
	PaymentsHistory,
	PaymentDetails {
		#[arg(short, long)]
		payment_id: String,
	},
	ListChannels,
	OpenChannel {
		#[arg(short, long)]
		node_id: String,
		#[arg(short, long)]
		address: String,
		#[arg(short, long)]
		channel_amount_sats: u64,
		#[arg(short, long)]
		push_to_counterparty_msat: Option<u64>,
		#[arg(long)]
		announce_channel: bool,
	},
	CloseChannel {
		#[arg(short, long)]
		user_channel_id: String,
		#[arg(short, long)]
		counterparty_node_id: String,
	},
	ForceCloseChannel {
		#[arg(short, long)]
		user_channel_id: String,
		#[arg(short, long)]
		counterparty_node_id: String,
	},
}

#[tokio::main]
async fn main() {
	let cli = Cli::parse();
	let client = ServerHackClient::new(cli.base_url);

	match cli.command {
		Commands::NodeId => {
			match client.get_node_id(GetNodeIdRequest {}).await {
				Ok(response) => {
					println!("Node ID: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting node ID: {:?}", e);
				},
			};
		},
		Commands::NodeStatus => {
			match client.get_node_status(GetNodeStatusRequest {}).await {
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
		Commands::PaymentDetails { payment_id } => {
			match client.get_payment_details(GetPaymentDetailsRequest { payment_id }).await {
				Ok(response) => {
					println!("Payment details: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting payment details: {:?}", e);
				},
			};
		},
		Commands::Bolt11Receive { description, expiry_secs, amount_msat } => {
			match client
				.bolt11_receive(Bolt11ReceiveRequest { description, expiry_secs, amount_msat })
				.await
			{
				Ok(response) => {
					println!("New invoice: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error getting invoice: {:?}", e);
				},
			};
		},
		Commands::Bolt11Send { invoice, amount_msat } => {
			match client.bolt11_send(Bolt11SendRequest { invoice, amount_msat }).await {
				Ok(response) => {
					println!("Sent BOLT11 payment: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error sending BOLT11 payment: {:?}", e);
				},
			};
		},
		Commands::OpenChannel {
			node_id,
			address,
			channel_amount_sats,
			push_to_counterparty_msat,
			announce_channel,
		} => {
			match client
				.open_channel(OpenChannelRequest {
					node_id,
					address,
					channel_amount_sats,
					push_to_counterparty_msat,
					announce_channel,
				})
				.await
			{
				Ok(response) => {
					println!("Open channel response: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error opening channel: {:?}", e);
				},
			};
		},
		Commands::CloseChannel { user_channel_id, counterparty_node_id } => {
			match client
				.close_channel(CloseChannelRequest {
					user_channel_id: user_channel_id.into_bytes(),
					counterparty_node_id,
				})
				.await
			{
				Ok(response) => {
					println!("Close channel response: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error closing channel: {:?}", e);
				},
			};
		},
		Commands::ForceCloseChannel { user_channel_id, counterparty_node_id } => {
			match client
				.force_close_channel(ForceCloseChannelRequest {
					user_channel_id: user_channel_id.into_bytes(),
					counterparty_node_id,
				})
				.await
			{
				Ok(response) => {
					println!("Force close channel response: {:?}", response);
				},
				Err(e) => {
					eprintln!("Error force closing channel: {:?}", e);
				},
			};
		},
	}
}
