use ldk_node::Node;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::Service;
use hyper::{Request, Response};

use core::future::Future;
use core::pin::Pin;
use std::sync::Arc;

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
		let msg = format!("{:?}", self.node.status());
		make_response(msg)
	}

	fn default_response(&self) -> <NodeService as Service<Request<Incoming>>>::Future {
		let msg = format!("UNKNOWN REQUEST");
		make_response(msg)
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

fn make_response(s: String) -> <NodeService as Service<Request<Incoming>>>::Future {
	Box::pin(async { Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap()) })
}
