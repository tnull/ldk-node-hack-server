mod error;

use crate::error::ServerHackError;
use prost::Message;

use protos::{
	GetBalancesRequest, GetBalancesResponse, GetNodeStatusRequest, GetNodeStatusResponse,
	ListChannelsRequest, OnchainReceiveRequest, OnchainReceiveResponse, OnchainSendRequest,
	OnchainSendResponse, PaymentsHistoryRequest, PaymentsHistoryResponse,
};
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";

#[derive(Clone)]
pub struct ServerHackClient {
	base_url: String,
	client: Client,
}

impl ServerHackClient {
	pub fn new(base_url: String) -> Self {
		Self { base_url, client: Client::new() }
	}

	pub async fn get_node_status(
		&self, request: &GetNodeStatusRequest,
	) -> Result<GetNodeStatusResponse, ServerHackError> {
		let url = format!("http://{}/getNodeStatus", self.base_url);
		self.post_request(request, &url).await
	}

	pub async fn get_new_funding_address(
		&self, request: OnchainReceiveRequest,
	) -> Result<OnchainReceiveResponse, ServerHackError> {
		let url = format!("http://{}//onchain/receive", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn send_onchain(
		&self, request: OnchainSendRequest,
	) -> Result<OnchainSendResponse, ServerHackError> {
		let url = format!("http://{}/onchain/send", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn get_node_balances(
		&self, request: GetBalancesRequest,
	) -> Result<GetBalancesResponse, ServerHackError> {
		let url = format!("http://{}/getNodeBalances", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn list_channels(
		&self, request: ListChannelsRequest,
	) -> Result<ListChannelsRequest, ServerHackError> {
		let url = format!("http://{}/listChannels", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn get_payments_history(
		&self, request: &PaymentsHistoryRequest,
	) -> Result<PaymentsHistoryResponse, ServerHackError> {
		let url = format!("http://{}/listPaymentsHistory", self.base_url);
		self.post_request(request, &url).await
	}

	async fn post_request<Rq: Message, Rs: Message + Default>(
		&self, request: &Rq, url: &str,
	) -> Result<Rs, ServerHackError> {
		let request_body = request.encode_to_vec();
		let response_raw = match self
			.client
			.post(url)
			.header(CONTENT_TYPE, APPLICATION_OCTET_STREAM)
			.body(request_body.clone())
			.send()
			.await
		{
			Ok(response) => response,
			Err(e) => {
				return Err(ServerHackError::InternalError(e.to_string()));
			},
		};
		let status = response_raw.status();
		let payload = response_raw.bytes().await?;

		if status.is_success() {
			Rs::decode(&payload[..]).map_err(|_| {
				ServerHackError::FailedRequest(
					reqwest::StatusCode::INTERNAL_SERVER_ERROR,
					String::from_utf8(payload.to_vec()).unwrap(),
				)
			})
		} else {
			Err(ServerHackError::FailedRequest(
				status,
				String::from_utf8(payload.to_vec()).unwrap(),
			))
		}
	}
}
