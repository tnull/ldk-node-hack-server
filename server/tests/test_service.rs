/// Integration tests for the service module.
/// Intentionally ignored as they require starting the servers which is slow.
/// Run with `cargo test --test test_service -- --ignored`
mod docker_compose;

use core::panic;
use std::{collections::HashMap, ptr::addr_of, sync::Once};

use client::ServerHackClient;
use docker_compose::{DockerCompose, DockerComposeConfig};
use protos::{
	Bolt11ReceiveRequest, CloseChannelRequest, ForceCloseChannelRequest, GetBalancesRequest,
	GetNodeIdRequest, GetNodeStatusRequest, GetPaymentDetailsRequest, ListChannelsRequest,
	OnchainReceiveRequest, OnchainSendRequest, OpenChannelRequest, PaymentsHistoryRequest,
};

const SERVER_URL: &str = "localhost:3000";
const COMPOSE_FILE_PATH: &str = "../docker-compose.yml";

fn start_docker_compose() -> DockerCompose {
	match DockerCompose::up_with_output(DockerComposeConfig {
		compose_file: COMPOSE_FILE_PATH.into(),
		env: HashMap::new(),
		project_name: "integration-tests".to_string(),
	}) {
		Ok((docker_compose, output)) => {
			println!("docker-compose output: {output:?}");
			docker_compose
		},
		Err(e) => {
			panic!("Error starting docker-compose: {e:?}");
		},
	}
}

#[tokio::test]
#[ignore]
async fn test_service_endpoints_dont_fail_with_sample_requests() {
	// Doing this in one test for now to easily start the servers once.
	// Can use something like once_cell later to split the tests up.
	// For now the tests do not interfere with each other but they probably would
	// with some actual testing of expected output.
	let docker_compose = start_docker_compose();

	let client = ServerHackClient::new(SERVER_URL.to_string());

	node_status_request(&client).await;
	get_node_id(&client).await;
	let address = onchain_receive(&client).await;
	// Failing with "The available funds are insufficient to complete the given operation.""
	// onchain_send(&client, address).await;
	bolt11_receive(&client).await;
	get_node_balances(&client).await;
	get_payments_history(&client).await;
	// Need bolt11_send to get a payment id
	// get_payment_details(&client, payment_id).await
	list_channels(&client).await;
	// Need functionality to add funds to open channels first
	// let channel_id = open_channel(&client).await;
	// close_channel(&client, channel_id.clone()).await;
	// force_close_channel(&client, channel_id).await;

	drop(docker_compose);
}

async fn node_status_request(client: &ServerHackClient) {
	let request = GetNodeStatusRequest {};
	client.get_node_status(request).await.unwrap();
}

async fn get_node_id(client: &ServerHackClient) {
	let request = GetNodeIdRequest {};
	client.get_node_id(request).await.unwrap();
}

async fn onchain_receive(client: &ServerHackClient) -> String {
	let request = OnchainReceiveRequest {};
	let response = client.get_new_funding_address(request).await.unwrap();
	response.address
}

async fn onchain_send(client: &ServerHackClient, address: String) {
	let request = OnchainSendRequest { address, amount_sats: Some(10_000) };
	client.send_onchain(request).await.unwrap();
}

async fn bolt11_receive(client: &ServerHackClient) {
	let request = Bolt11ReceiveRequest {
		description: "description".to_string(),
		expiry_secs: 10,
		amount_msat: Some(10_000),
	};
	client.bolt11_receive(request).await.unwrap();
}

async fn get_node_balances(client: &ServerHackClient) {
	let request = GetBalancesRequest {};
	client.get_node_balances(request).await.unwrap();
}

async fn get_payments_history(client: &ServerHackClient) {
	let request = PaymentsHistoryRequest {};
	client.get_payments_history(request).await.unwrap();
}

async fn get_payment_details(client: &ServerHackClient, payment_id: String) {
	// To get a valid payment id, use bolt11_recieve to get and invoice and then bolt11_send to that invoice .
	let request = GetPaymentDetailsRequest { payment_id };
	client.get_payment_details(request).await.unwrap();
}

async fn list_channels(client: &ServerHackClient) {
	let request = ListChannelsRequest {};
	client.list_channels(request).await.unwrap();
}

async fn open_channel(client: &ServerHackClient) -> Vec<u8> {
	let request = OpenChannelRequest {
		node_id: "027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190".to_string(),
		address: "localhost:3042".to_string(),
		channel_amount_sats: 100_000,
		push_to_counterparty_msat: None,
		announce_channel: true,
	};
	let response = client.open_channel(request).await.unwrap();
	response.user_channel_id
}

async fn close_channel(client: &ServerHackClient, channel_id: Vec<u8>) {
	let request = CloseChannelRequest {
		user_channel_id: channel_id,
		counterparty_node_id: "027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190"
			.to_string(),
	};
	client.close_channel(request).await.unwrap();
}

async fn force_close_channel(client: &ServerHackClient, channel_id: Vec<u8>) {
	let request = ForceCloseChannelRequest {
		user_channel_id: channel_id,
		counterparty_node_id: "027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190"
			.to_string(),
	};
	client.force_close_channel(request).await.unwrap();
}
