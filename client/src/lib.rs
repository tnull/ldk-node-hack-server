mod error;

use crate::error::ServerHackError;
use prost::Message;

use protos::{
	Bolt11ReceiveRequest, Bolt11ReceiveResponse, GetBalancesRequest, GetBalancesResponse,
	GetNodeStatusRequest, GetNodeStatusResponse, GetPaymentDetailsRequest,
	GetPaymentDetailsResponse, ListChannelsRequest, ListChannelsResponse, OnchainReceiveRequest,
	OnchainReceiveResponse, OnchainSendRequest, OnchainSendResponse, PaymentsHistoryRequest,
	PaymentsHistoryResponse,
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
		let url = format!("http://{}/onchain/receive", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn send_onchain(
		&self, request: OnchainSendRequest,
	) -> Result<OnchainSendResponse, ServerHackError> {
		let url = format!("http://{}/onchain/send", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn bolt11_receive(
		&self, request: Bolt11ReceiveRequest,
	) -> Result<Bolt11ReceiveResponse, ServerHackError> {
		let url = format!("http://{}/bolt11/receive", self.base_url);
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
	) -> Result<ListChannelsResponse, ServerHackError> {
		let url = format!("http://{}/listChannels", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn get_payments_history(
		&self, request: &PaymentsHistoryRequest,
	) -> Result<PaymentsHistoryResponse, ServerHackError> {
		let url = format!("http://{}/listPaymentsHistory", self.base_url);
		self.post_request(request, &url).await
	}

	pub async fn get_payment_details(
		&self, request: GetPaymentDetailsRequest,
	) -> Result<GetPaymentDetailsResponse, ServerHackError> {
		let url = format!("http://{}/getPaymentDetails", self.base_url);
		self.post_request(&request, &url).await
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
		let payload = response_raw.text().await?;

		if status.is_success() {
			Rs::decode(&payload.encode_to_vec()[..]).map_err(|_| {
				ServerHackError::FailedRequest(
					reqwest::StatusCode::INTERNAL_SERVER_ERROR,
                    payload,
				)
			})
		} else {
			Err(ServerHackError::FailedRequest(
				status,
                    payload,
			))
		}
	}
}
