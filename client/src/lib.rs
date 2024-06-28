mod error;

use crate::error::ServerHackError;
use prost::Message;

use protos::{
	Bolt11ReceiveRequest, Bolt11ReceiveResponse, Bolt11SendRequest, Bolt11SendResponse,
	Bolt12ReceiveRequest, Bolt12ReceiveResponse, CloseChannelRequest, CloseChannelResponse,
	ForceCloseChannelRequest, ForceCloseChannelResponse, GetBalancesRequest, GetBalancesResponse,
	GetNodeIdRequest, GetNodeIdResponse, GetNodeStatusRequest, GetNodeStatusResponse,
	GetPaymentDetailsRequest, GetPaymentDetailsResponse, ListChannelsRequest, ListChannelsResponse,
	OnchainReceiveRequest, OnchainReceiveResponse, OnchainSendRequest, OnchainSendResponse,
	OpenChannelRequest, OpenChannelResponse, PaymentsHistoryRequest, PaymentsHistoryResponse,
};
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";

const GET_NODE_ID_PATH: &str = "getNodeId";
const GET_NODE_STATUS_PATH: &str = "getNodeStatus";
const ONCHAIN_RECEIVE_PATH: &str = "onchain/receive";
const ONCHAIN_SEND_PATH: &str = "onchain/send";
const BOLT11_RECEIVE_PATH: &str = "bolt11/receive";
const BOLT11_SEND_PATH: &str = "bolt11/send";
const BOLT12_RECEIVE_PATH: &str = "bolt12/receive";
const GET_NODE_BALANCES_PATH: &str = "getNodeBalances";
const PAYMENTS_HISTORY_PATH: &str = "listPaymentsHistory";
const GET_PAYMENT_DETAILS_PATH: &str = "getPaymentDetails";
const LIST_CHANNELS_PATH: &str = "channel/list";
const OPEN_CHANNEL_PATH: &str = "channel/open";
const CLOSE_CHANNEL_PATH: &str = "channel/close";
const FORCE_CLOSE_CHANNEL_PATH: &str = "channel/force-close";

#[derive(Clone)]
pub struct ServerHackClient {
	base_url: String,
	client: Client,
}

impl ServerHackClient {
	pub fn new(base_url: String) -> Self {
		Self { base_url, client: Client::new() }
	}

	pub async fn get_node_id(
		&self, request: GetNodeIdRequest,
	) -> Result<GetNodeIdResponse, ServerHackError> {
		let url = format!("http://{}/{GET_NODE_ID_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn get_node_status(
		&self, request: GetNodeStatusRequest,
	) -> Result<GetNodeStatusResponse, ServerHackError> {
		let url = format!("http://{}/{GET_NODE_STATUS_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn get_new_funding_address(
		&self, request: OnchainReceiveRequest,
	) -> Result<OnchainReceiveResponse, ServerHackError> {
		let url = format!("http://{}/{ONCHAIN_RECEIVE_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn send_onchain(
		&self, request: OnchainSendRequest,
	) -> Result<OnchainSendResponse, ServerHackError> {
		let url = format!("http://{}/{ONCHAIN_SEND_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn bolt11_receive(
		&self, request: Bolt11ReceiveRequest,
	) -> Result<Bolt11ReceiveResponse, ServerHackError> {
		let url = format!("http://{}/{BOLT11_RECEIVE_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn bolt11_send(
		&self, request: Bolt11SendRequest,
	) -> Result<Bolt11SendResponse, ServerHackError> {
		let url = format!("http://{}/{BOLT11_SEND_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn bolt12_receive(
		&self, request: Bolt12ReceiveRequest,
	) -> Result<Bolt12ReceiveResponse, ServerHackError> {
		let url = format!("http://{}/{BOLT12_RECEIVE_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn get_node_balances(
		&self, request: GetBalancesRequest,
	) -> Result<GetBalancesResponse, ServerHackError> {
		let url = format!("http://{}/{GET_NODE_BALANCES_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn list_channels(
		&self, request: ListChannelsRequest,
	) -> Result<ListChannelsResponse, ServerHackError> {
		let url = format!("http://{}/{LIST_CHANNELS_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn get_payments_history(
		&self, request: &PaymentsHistoryRequest,
	) -> Result<PaymentsHistoryResponse, ServerHackError> {
		let url = format!("http://{}/{PAYMENTS_HISTORY_PATH}", self.base_url);
		self.post_request(request, &url).await
	}

	pub async fn get_payment_details(
		&self, request: GetPaymentDetailsRequest,
	) -> Result<GetPaymentDetailsResponse, ServerHackError> {
		let url = format!("http://{}/{GET_PAYMENT_DETAILS_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn open_channel(
		&self, request: OpenChannelRequest,
	) -> Result<OpenChannelResponse, ServerHackError> {
		let url = format!("http://{}/{OPEN_CHANNEL_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn close_channel(
		&self, request: CloseChannelRequest,
	) -> Result<CloseChannelResponse, ServerHackError> {
		let url = format!("http://{}/{CLOSE_CHANNEL_PATH}", self.base_url);
		self.post_request(&request, &url).await
	}

	pub async fn force_close_channel(
		&self, request: ForceCloseChannelRequest,
	) -> Result<ForceCloseChannelResponse, ServerHackError> {
		let url = format!("http://{}/{FORCE_CLOSE_CHANNEL_PATH}", self.base_url);
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
