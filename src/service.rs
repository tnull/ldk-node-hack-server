use ldk_node::lightning::chain::BestBlock;
use ldk_node::Node;
use prost::Message;

use core::future::Future;
use core::pin::Pin;
use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::Service;
use hyper::{Request, Response};
use std::sync::Arc;

use protos::GetNodeStatusResponse;

const GET_NODE_STATUS_PATH: &str = "/status";

type Req = Request<Incoming>;

#[derive(Clone)]
pub struct NodeService {
	node: Arc<Node>,
}

impl NodeService {
	pub(crate) fn new(node: Arc<Node>) -> Self {
		Self { node }
	}

	fn handle_get_node_status_request(
		&self, _: Req,
	) -> <NodeService as Service<Request<Incoming>>>::Future {
		let status = self.node.status();
		let BestBlock { block_hash, height } = status.current_best_block;

		let msg = GetNodeStatusResponse {
			public_key: self.node.node_id().to_string(),
			current_best_block: Some(protos::BestBlock {
				block_hash: block_hash.to_string(),
				height,
			}),
			latest_wallet_sync_timestamp: status.latest_wallet_sync_timestamp,
			latest_onchain_wallet_sync_timestamp: status.latest_onchain_wallet_sync_timestamp,
			latest_fee_rate_cache_update_timestamp: status.latest_fee_rate_cache_update_timestamp,
			latest_rgs_snapshot_timestamp: status.latest_rgs_snapshot_timestamp,
			latest_node_announcement_broadcast_timestamp: status
				.latest_node_announcement_broadcast_timestamp,
		};
		make_response(msg.encode_to_vec())
	}

	fn default_response(&self) -> <NodeService as Service<Request<Incoming>>>::Future {
		make_response(b"UNKNOWN REQUEST".to_vec())
	}
}

impl Service<Req> for NodeService {
	type Response = Response<Full<Bytes>>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Req) -> Self::Future {
		println!("processing request: {} {}", req.method(), req.uri().path());
		match req.uri().path() {
			GET_NODE_STATUS_PATH => self.handle_get_node_status_request(req),
			_ => self.default_response(),
		}
	}
}

fn make_response(bytes: Vec<u8>) -> <NodeService as Service<Request<Incoming>>>::Future {
	Box::pin(async { Ok(Response::builder().body(Full::new(bytes.into())).unwrap()) })
}
