pub mod config;

use std::sync::Arc;

use tokio::signal::unix::SignalKind;

use ldk_node::{Builder, Config as LdkNodeConfig, Event};

use config::Config;

fn main() {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 2 {
		eprintln!("Usage: {} storage_path", args[0]);
		std::process::exit(-1);
	}

	let config = Config::new(&args[1]).unwrap();

	let mut ldk_node_config = LdkNodeConfig::default();
	ldk_node_config.storage_dir_path = args[1];
	ldk_node_config.log_level = config.log_level;
	ldk_node_config.network = config.network;
	ldk_node_config.listening_addresses = Some(vec![config.listening_addr]);

	let mut builder = Builder::from_config(ldk_node_config);
	builder.set_esplora_server(config.esplora_server_url);

	let runtime =
		Arc::new(tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap());

	let node = Arc::new(builder.build().unwrap());
	println!("Starting up...");
	node.start_with_runtime(Arc::clone(&runtime)).unwrap();

	println!("CONNECTION_STRING: {}@{}", node.node_id(), config.listening_addr,);

	runtime.block_on(async {
		let mut sigterm_stream = match tokio::signal::unix::signal(SignalKind::terminate()) {
			Ok(stream) => stream,
			Err(e) => {
				println!("Failed to register for SIGTERM stream: {}", e);
				std::process::exit(-1);
			},
		};
		let event_node = Arc::clone(&node);
		loop {
			tokio::select! {
				event = event_node.next_event_async() => {
					match event {
						Event::ChannelPending { channel_id, counterparty_node_id, .. } => {
							println!(
								"CHANNEL_PENDING: {} from counterparty {}",
								channel_id, counterparty_node_id
								);
						},
						Event::ChannelReady { channel_id, counterparty_node_id, .. } => {
							println!(
								"CHANNEL_READY: {} from counterparty {:?}",
								channel_id, counterparty_node_id
								);
						},
						Event::PaymentReceived { payment_id, payment_hash, amount_msat } => {
							println!(
								"PAYMENT_RECEIVED: with id {:?}, hash {}, amount_msat {}",
								payment_id, payment_hash, amount_msat
								);
						},
						_ => {},
					}
					event_node.event_handled();
				},
				_ = tokio::signal::ctrl_c() => {
					println!("Received CTRL-C, shutting down..");
					break;
				}
				_ = sigterm_stream.recv() => {
					println!("Received SIGTERM, shutting down..");
					break;
				}
			}
		}
	});

	node.stop().unwrap();
	println!("Shutdown complete..");
}
