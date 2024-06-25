use ldk_node::Node;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::service::Service;
use hyper::{Request, Response};

use core::future::Future;
use core::pin::Pin;
use std::sync::Arc;

type Req = Request<Incoming>;

#[derive(Clone)]
pub struct NodeService {
	node: Arc<Node>,
}

impl NodeService {
	pub(crate) fn new(node: Arc<Node>) -> Self {
		Self { node }
	}

	// USE: pub(crate) async fn handle_request(&self, _: Req) -> Result<Response<Full<Bytes>>, Infallible> {
	//	Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
	//}

	fn default_response(&self) -> <NodeService as Service<Request<Incoming>>>::Future {
		let msg = format!("UNKNOWN REQUEST. Status: {:?}", self.node.status());
		make_response(msg)
	}
}

impl Service<Req> for NodeService {
	type Response = Response<Full<Bytes>>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Req) -> Self::Future {
		println!("processing request: {} {}", req.method(), req.uri().path());
		self.default_response()
	}
}

fn make_response(s: String) -> <NodeService as Service<Request<Incoming>>>::Future {
	Box::pin(async { Ok(Response::builder().body(Full::new(Bytes::from(s))).unwrap()) })
}
