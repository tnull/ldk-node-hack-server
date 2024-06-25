mod config;
mod utils;

use std::{path::Path, sync::Arc};

use tokio::signal::unix::SignalKind;

use ldk_node::{Builder, Config as LdkNodeConfig, Event};

const CONFIG_FILE_NAME: &str = "config.json";

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use crate::service::NodeService;

mod service;

fn main() {
	let args: Vec<String> = std::env::args().collect();

	if args.len() < 2 {
		eprintln!("Usage: {} storage_path", args[0]);
		std::process::exit(-1);
	}

	let config = utils::read_config_from_json(Path::new(&args[1]).join(CONFIG_FILE_NAME)).unwrap();

	let mut ldk_node_config = LdkNodeConfig::default();
	ldk_node_config.storage_dir_path = args[1].clone();
	ldk_node_config.log_level = config.log_level;
	ldk_node_config.network = config.network;
	ldk_node_config.listening_addresses = Some(vec![config.listening_addr.clone()]);

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
		let rest_svc_listener = TcpListener::bind(config.rest_service_addr)
			.await
			.expect("Failed to bind listening port");
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
				res = rest_svc_listener.accept() => {
					match res {
						Ok((stream, _)) => {
							let io_stream = TokioIo::new(stream);
							let node_service = NodeService::new(Arc::clone(&node));
							runtime.spawn(async move {
								if let Err(err) = http1::Builder::new().serve_connection(io_stream, node_service).await {
									eprintln!("Failed to serve connection: {}", err);
								}
							});
						},
						Err(e) => eprintln!("Failed to accept connection: {}", e),
					}
				}
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
