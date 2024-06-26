mod error;

use crate::error::ServerHackError;
use crate::proto::ldk_server_hack::GetNodeStatusResponse;
use prost::Message;
use reqwest::header::CONTENT_TYPE;
use reqwest::Client;

const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";

pub mod proto {
	pub mod ldk_server_hack {
		tonic::include_proto!("ldk_server_hack");
	}
}

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
		&self, _request: proto::ldk_server_hack::GetNodeStatusRequest,
	) -> Result<GetNodeStatusResponse, ServerHackError> {
		let url = format!("http://{}/status", self.base_url);
		self.post_request(&_request, &url).await
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
			let response = Rs::decode(&payload[..])?;
			Ok(response)
		} else {
			Err(ServerHackError::FailedRequest(status, payload.to_vec()))
		}
	}
}
