use ldk_node::bitcoin::secp256k1::PublicKey;
use ldk_node::bitcoin::Address;
use ldk_node::lightning::chain::BestBlock;
use ldk_node::lightning::ln::msgs::SocketAddress;
use ldk_node::lightning::offers::offer::Offer;
use ldk_node::lightning_invoice::Bolt11Invoice;
use ldk_node::payment::{PaymentDetails, PaymentDirection, PaymentKind, PaymentStatus};
use ldk_node::LightningBalance::{
	ClaimableAwaitingConfirmations, ClaimableOnChannelClose, ContentiousClaimable,
	CounterpartyRevokedOutputClaimable, MaybePreimageClaimableHTLC, MaybeTimeoutClaimableHTLC,
};
use ldk_node::PendingSweepBalance::{
	AwaitingThresholdConfirmations, BroadcastAwaitingConfirmation, PendingBroadcast,
};
use ldk_node::{Node, UserChannelId};
use prost::Message;

use core::future::Future;
use core::pin::Pin;
use core::str::FromStr;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::Service;
use hyper::{Request, Response, StatusCode};

use std::sync::Arc;

use protos::{
	lightning_balance, pending_sweep_balance, Bolt11ReceiveRequest, Bolt11ReceiveResponse,
	Bolt11SendRequest, Bolt11SendResponse, Bolt12ReceiveRequest, Bolt12ReceiveResponse,
	Bolt12SendRequest, Bolt12SendResponse, Channel, CloseChannelRequest, CloseChannelResponse,
	ForceCloseChannelRequest, ForceCloseChannelResponse, GetBalancesRequest, GetBalancesResponse,
	GetNodeIdRequest, GetNodeIdResponse, GetNodeStatusRequest, GetNodeStatusResponse,
	GetPaymentDetailsRequest, ListChannelsRequest, ListChannelsResponse, OnchainReceiveRequest,
	OnchainReceiveResponse, OnchainSendRequest, OnchainSendResponse, OpenChannelRequest,
	OpenChannelResponse, Outpoint, PaymentsHistoryRequest, PaymentsHistoryResponse,
};

const GET_NODE_ID_PATH: &str = "/getNodeId";
const GET_NODE_STATUS_PATH: &str = "/getNodeStatus";
const ONCHAIN_RECEIVE_PATH: &str = "/onchain/receive";
const ONCHAIN_SEND_PATH: &str = "/onchain/send";
const BOLT11_RECEIVE_PATH: &str = "/bolt11/receive";
const BOLT11_SEND_PATH: &str = "/bolt11/send";
const BOLT12_RECEIVE_PATH: &str = "/bolt12/receive";
const BOLT12_SEND_PATH: &str = "/bolt12/send";
const GET_NODE_BALANCES_PATH: &str = "/getNodeBalances";
const PAYMENTS_HISTORY_PATH: &str = "/listPaymentsHistory";
const GET_PAYMENT_DETAILS_PATH: &str = "/getPaymentDetails";
const LIST_CHANNELS_PATH: &str = "/channel/list";
const OPEN_CHANNEL_PATH: &str = "/channel/open";
const CLOSE_CHANNEL_PATH: &str = "/channel/close";
const FORCE_CLOSE_CHANNEL_PATH: &str = "/channel/force-close";

type Req = Request<Incoming>;

#[derive(Clone)]
pub struct NodeService {
	node: Arc<Node>,
}

impl NodeService {
	pub(crate) fn new(node: Arc<Node>) -> Self {
		Self { node }
	}
}

impl Service<Req> for NodeService {
	type Response = Response<Full<Bytes>>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Req) -> Self::Future {
		println!("processing request: {} {}", req.method(), req.uri().path());
		let node = Arc::clone(&self.node);
		match req.uri().path() {
			GET_NODE_ID_PATH => Box::pin(handle_request(node, req, handle_get_node_id_request)),
			GET_NODE_STATUS_PATH => {
				Box::pin(handle_request(node, req, handle_get_node_status_request))
			},
			GET_NODE_BALANCES_PATH => {
				Box::pin(handle_request(node, req, handle_get_balances_request))
			},
			ONCHAIN_RECEIVE_PATH => Box::pin(handle_request(node, req, handle_onchain_receive)),
			ONCHAIN_SEND_PATH => Box::pin(handle_request(node, req, handle_onchain_send)),
			BOLT11_RECEIVE_PATH => {
				Box::pin(handle_request(node, req, handle_bolt11_receive_request))
			},
			BOLT11_SEND_PATH => Box::pin(handle_request(node, req, handle_bolt11_send_request)),
			BOLT12_RECEIVE_PATH => {
				Box::pin(handle_request(node, req, handle_bolt12_receive_request))
			},
			BOLT12_SEND_PATH => Box::pin(handle_request(node, req, handle_bolt12_send_request)),
			LIST_CHANNELS_PATH => Box::pin(handle_request(node, req, handle_list_channels_request)),
			OPEN_CHANNEL_PATH => Box::pin(handle_request(node, req, handle_open_channel)),
			CLOSE_CHANNEL_PATH => Box::pin(handle_request(node, req, handle_close_channel)),
			FORCE_CLOSE_CHANNEL_PATH => {
				Box::pin(handle_request(node, req, handle_force_close_channel))
			},
			PAYMENTS_HISTORY_PATH => {
				Box::pin(handle_request(node, req, handle_get_payment_history_request))
			},
			GET_PAYMENT_DETAILS_PATH => {
				Box::pin(handle_request(node, req, handle_get_payment_details_request))
			},
			path => {
				let error = format!("Unknown request: {}", path).into_bytes();
				Box::pin(async {
					Ok(Response::builder()
						.status(StatusCode::BAD_REQUEST)
						.body(Full::new(Bytes::from(error)))
						.unwrap())
				})
			},
		}
	}
}

async fn handle_request<
	T: Message + Default,
	R: Message,
	F: Fn(Arc<Node>, T) -> Result<R, ldk_node::NodeError>,
>(
	node: Arc<Node>, request: Req, handler: F,
) -> Result<<NodeService as Service<Request<Incoming>>>::Response, hyper::Error> {
	let bytes = request.into_body().collect().await?.to_bytes();
	match T::decode(bytes) {
		Ok(request) => match handler(node, request) {
			Ok(response) => Ok(Response::builder()
				.body(Full::new(Bytes::from(response.encode_to_vec())))
				.unwrap()),
			Err(e) => Ok(Response::builder()
				.status(StatusCode::INTERNAL_SERVER_ERROR)
				.body(Full::new(Bytes::from(e.to_string().into_bytes())))
				.unwrap()),
		},
		Err(_) => Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(Full::new(Bytes::from(b"Error parsing request".to_vec())))
			.unwrap()),
	}
}

fn handle_get_node_id_request(
	node: Arc<Node>, _request: GetNodeIdRequest,
) -> Result<GetNodeIdResponse, ldk_node::NodeError> {
	let node_id = node.node_id();
	let response = GetNodeIdResponse { node_id: node_id.to_string() };
	Ok(response)
}

fn handle_get_node_status_request(
	node: Arc<Node>, _request: GetNodeStatusRequest,
) -> Result<GetNodeStatusResponse, ldk_node::NodeError> {
	let status = node.status();
	let BestBlock { block_hash, height } = status.current_best_block;

	let response = GetNodeStatusResponse {
		public_key: node.node_id().to_string(),
		current_best_block: Some(protos::BestBlock { block_hash: block_hash.to_string(), height }),
		latest_wallet_sync_timestamp: status.latest_wallet_sync_timestamp,
		latest_onchain_wallet_sync_timestamp: status.latest_onchain_wallet_sync_timestamp,
		latest_fee_rate_cache_update_timestamp: status.latest_fee_rate_cache_update_timestamp,
		latest_rgs_snapshot_timestamp: status.latest_rgs_snapshot_timestamp,
		latest_node_announcement_broadcast_timestamp: status
			.latest_node_announcement_broadcast_timestamp,
	};
	Ok(response)
}

fn handle_get_payment_history_request(
	node: Arc<Node>, _request: PaymentsHistoryRequest,
) -> Result<PaymentsHistoryResponse, ldk_node::NodeError> {
	let payments = node.list_payments();
	let response = protos::PaymentsHistoryResponse {
		payments: payments.iter().map(to_payment_details_proto).collect(),
	};
	Ok(response)
}

fn handle_onchain_receive(
	node: Arc<Node>, _request: OnchainReceiveRequest,
) -> Result<OnchainReceiveResponse, ldk_node::NodeError> {
	let response =
		OnchainReceiveResponse { address: node.onchain_payment().new_address()?.to_string() };
	Ok(response)
}

fn handle_onchain_send(
	node: Arc<Node>, request: OnchainSendRequest,
) -> Result<OnchainSendResponse, ldk_node::NodeError> {
	let address = Address::from_str(&request.address)
		.map_err(|_| ldk_node::NodeError::InvalidAddress)?
		.require_network(node.config().network)
		.map_err(|_| ldk_node::NodeError::InvalidAddress)?;
	let txid = match request.amount_sats {
		Some(amount_sats) => node.onchain_payment().send_to_address(&address, amount_sats)?,
		None => node.onchain_payment().send_all_to_address(&address)?,
	};
	let response = OnchainSendResponse { txid: txid.to_string() };
	Ok(response)
}

fn handle_get_balances_request(
	node: Arc<Node>, _request: GetBalancesRequest,
) -> Result<GetBalancesResponse, ldk_node::NodeError> {
	let balance_details = node.list_balances();
	let lightning_balances = balance_details
		.lightning_balances
		.into_iter()
		.map(|lightning_balance| match lightning_balance {
			ClaimableOnChannelClose { channel_id, counterparty_node_id, amount_satoshis } => {
				protos::LightningBalance {
					balance_type: Some(lightning_balance::BalanceType::ClaimableOnChannelClose(
						protos::ClaimableOnChannelClose {
							channel_id: channel_id.to_string(),
							counterparty_node_id: counterparty_node_id.to_string(),
							amount_satoshis,
						},
					)),
				}
			},
			ClaimableAwaitingConfirmations {
				channel_id,
				counterparty_node_id,
				amount_satoshis,
				confirmation_height,
			} => protos::LightningBalance {
				balance_type: Some(lightning_balance::BalanceType::ClaimableAwaitingConfirmations(
					protos::ClaimableAwaitingConfirmations {
						channel_id: channel_id.to_string(),
						counterparty_node_id: counterparty_node_id.to_string(),
						amount_satoshis,
						confirmation_height,
					},
				)),
			},
			ContentiousClaimable {
				channel_id,
				counterparty_node_id,
				amount_satoshis,
				timeout_height,
				payment_hash,
				payment_preimage,
			} => protos::LightningBalance {
				balance_type: Some(lightning_balance::BalanceType::ContentiousClaimable(
					protos::ContentiousClaimable {
						channel_id: channel_id.to_string(),
						counterparty_node_id: counterparty_node_id.to_string(),
						amount_satoshis,
						timeout_height,
						payment_hash: payment_hash.to_string(),
						payment_preimage: payment_preimage.to_string(),
					},
				)),
			},
			MaybeTimeoutClaimableHTLC {
				channel_id,
				counterparty_node_id,
				amount_satoshis,
				claimable_height,
				payment_hash,
			} => protos::LightningBalance {
				balance_type: Some(lightning_balance::BalanceType::MaybeTimeoutClaimableHtlc(
					protos::MaybeTimeoutClaimableHtlc {
						channel_id: channel_id.to_string(),
						counterparty_node_id: counterparty_node_id.to_string(),
						amount_satoshis,
						claimable_height,
						payment_hash: payment_hash.to_string(),
					},
				)),
			},
			MaybePreimageClaimableHTLC {
				channel_id,
				counterparty_node_id,
				amount_satoshis,
				expiry_height,
				payment_hash,
			} => protos::LightningBalance {
				balance_type: Some(lightning_balance::BalanceType::MaybePreimageClaimableHtlc(
					protos::MaybePreimageClaimableHtlc {
						channel_id: channel_id.to_string(),
						counterparty_node_id: counterparty_node_id.to_string(),
						amount_satoshis,
						expiry_height,
						payment_hash: payment_hash.to_string(),
					},
				)),
			},
			CounterpartyRevokedOutputClaimable {
				channel_id,
				counterparty_node_id,
				amount_satoshis,
			} => protos::LightningBalance {
				balance_type: Some(
					lightning_balance::BalanceType::CounterpartyRevokedOutputClaimable(
						protos::CounterpartyRevokedOutputClaimable {
							channel_id: channel_id.to_string(),
							counterparty_node_id: counterparty_node_id.to_string(),
							amount_satoshis,
						},
					),
				),
			},
		})
		.collect();
	let pending_balances_from_channel_closures = balance_details
		.pending_balances_from_channel_closures
		.into_iter()
		.map(|pending_sweep_balance| match pending_sweep_balance {
			PendingBroadcast { channel_id, amount_satoshis } => protos::PendingSweepBalance {
				balance_type: Some(pending_sweep_balance::BalanceType::PendingBroadcast(
					protos::PendingBroadcast {
						channel_id: channel_id.map(|id| id.to_string()).unwrap(),
						amount_satoshis,
					},
				)),
			},
			BroadcastAwaitingConfirmation {
				channel_id,
				latest_broadcast_height,
				latest_spending_txid,
				amount_satoshis,
			} => protos::PendingSweepBalance {
				balance_type: Some(
					pending_sweep_balance::BalanceType::BroadcastAwaitingConfirmation(
						protos::BroadcastAwaitingConfirmation {
							channel_id: channel_id.map(|id| id.to_string()),
							latest_broadcast_height,
							latest_spending_txid: latest_spending_txid.to_string(),
							amount_satoshis,
						},
					),
				),
			},
			AwaitingThresholdConfirmations {
				channel_id,
				latest_spending_txid,
				confirmation_hash,
				confirmation_height,
				amount_satoshis,
			} => protos::PendingSweepBalance {
				balance_type: Some(
					pending_sweep_balance::BalanceType::AwaitingThresholdConfirmations(
						protos::AwaitingThresholdConfirmations {
							channel_id: channel_id.map(|id| id.to_string()),
							latest_spending_txid: latest_spending_txid.to_string(),
							confirmation_hash: confirmation_hash.to_string(),
							confirmation_height,
							amount_satoshis,
						},
					),
				),
			},
		})
		.collect();
	let response = GetBalancesResponse {
		total_onchain_balance_sats: balance_details.total_onchain_balance_sats,
		spendable_onchain_balance_sats: balance_details.spendable_onchain_balance_sats,
		total_anchor_channels_reserve_sats: balance_details.total_anchor_channels_reserve_sats,
		total_lightning_balance_sats: balance_details.total_lightning_balance_sats,
		lightning_balances,
		pending_balances_from_channel_closures,
	};
	Ok(response)
}

fn handle_bolt11_receive_request(
	node: Arc<Node>, request: Bolt11ReceiveRequest,
) -> Result<Bolt11ReceiveResponse, ldk_node::NodeError> {
	let invoice = match request.amount_msat {
		Some(amount_msat) => {
			node.bolt11_payment().receive(amount_msat, &request.description, request.expiry_secs)?
		},
		None => node
			.bolt11_payment()
			.receive_variable_amount(&request.description, request.expiry_secs)?,
	};

	let response = Bolt11ReceiveResponse { invoice: invoice.to_string() };
	Ok(response)
}

fn handle_bolt11_send_request(
	node: Arc<Node>, request: Bolt11SendRequest,
) -> Result<Bolt11SendResponse, ldk_node::NodeError> {
	let invoice = Bolt11Invoice::from_str(&request.invoice)
		.map_err(|_| ldk_node::NodeError::InvalidInvoice)?;
	let payment_id = match request.amount_msat {
		Some(amount_msat) => node.bolt11_payment().send_using_amount(&invoice, amount_msat)?,
		None => node.bolt11_payment().send(&invoice)?,
	};

	let response =
		Bolt11SendResponse { payment_id: Some(protos::PaymentId { data: payment_id.0.to_vec() }) };
	Ok(response)
}

fn handle_bolt12_receive_request(
	node: Arc<Node>, request: Bolt12ReceiveRequest,
) -> Result<Bolt12ReceiveResponse, ldk_node::NodeError> {
	let offer = match request.amount_msat {
		Some(amount_msat) => node.bolt12_payment().receive(amount_msat, &request.description)?,
		None => node.bolt12_payment().receive_variable_amount(&request.description)?,
	};

	let response = Bolt12ReceiveResponse { offer: offer.to_string() };
	Ok(response)
}

fn handle_bolt12_send_request(
	node: Arc<Node>, request: Bolt12SendRequest,
) -> Result<Bolt12SendResponse, ldk_node::NodeError> {
	let offer = Offer::from_str(&request.offer).map_err(|_| ldk_node::NodeError::InvalidInvoice)?;
	let payment_id = match request.amount_msat {
		Some(amount_msat) => {
			node.bolt12_payment().send_using_amount(&offer, request.payer_note, amount_msat)?
		},
		None => node.bolt12_payment().send(&offer, request.payer_note)?,
	};

	let response =
		Bolt12SendResponse { payment_id: Some(protos::PaymentId { data: payment_id.0.to_vec() }) };
	Ok(response)
}

fn handle_list_channels_request(
	node: Arc<Node>, _request: ListChannelsRequest,
) -> Result<ListChannelsResponse, ldk_node::NodeError> {
	let channels = node
		.list_channels()
		.iter()
		.map(|c| Channel {
			channel_id: c.channel_id.to_string(),
			counterparty_node_id: c.counterparty_node_id.to_string(),
			funding_txo: c.funding_txo.map(|o| Outpoint { txid: o.txid.to_string(), vout: o.vout }),
			channel_value_sats: c.channel_value_sats,
			feerate_sat_per_1000_weight: c.feerate_sat_per_1000_weight,
			outbound_capacity_msat: c.outbound_capacity_msat,
			inbound_capacity_msat: c.inbound_capacity_msat,
			confirmations_required: c.confirmations_required,
			confirmations: c.confirmations,
			is_outbound: c.is_outbound,
			is_channel_ready: c.is_channel_ready,
			is_usable: c.is_usable,
			is_public: c.is_public,
			cltv_expiry_delta: c.cltv_expiry_delta.map(|cltv| cltv as u32),
			counterparty_outbound_htlc_minimum_msat: c.counterparty_outbound_htlc_minimum_msat,
			counterparty_outbound_htlc_maximum_msat: c.counterparty_outbound_htlc_maximum_msat,
			next_outbound_htlc_limit_msat: c.next_outbound_htlc_limit_msat,
			next_outbound_htlc_minimum_msat: c.next_outbound_htlc_minimum_msat,
			force_close_spend_delay: c.force_close_spend_delay.map(|delay| delay as u32),
			forwarding_fee_proportional_millionths: c
				.config
				.forwarding_fee_proportional_millionths(),
			forwarding_fee_base_msat: c.config.forwarding_fee_base_msat(),
		})
		.collect();

	let response = ListChannelsResponse { channels };
	Ok(response)
}

fn handle_get_payment_details_request(
	node: Arc<Node>, request: GetPaymentDetailsRequest,
) -> Result<protos::PaymentDetails, ldk_node::NodeError> {
	let payment_id = request.payment_id.as_bytes();
	if payment_id.len() != 32 {
		return Err(ldk_node::NodeError::InvalidPaymentId);
	}

	let mut arr = [0u8; 32];
	arr.copy_from_slice(&payment_id[..]);
	let payment_id = ldk_node::lightning::ln::channelmanager::PaymentId(arr);
	if let Some(payment_details) = node.payment(&payment_id) {
		let response = to_payment_details_proto(&payment_details);
		return Ok(response);
	}

	return Err(ldk_node::NodeError::InvalidPaymentId);
}

fn handle_open_channel(
	node: Arc<Node>, request: OpenChannelRequest,
) -> Result<OpenChannelResponse, ldk_node::NodeError> {
	let node_id =
		PublicKey::from_str(&request.node_id).map_err(|_| ldk_node::NodeError::InvalidNodeId)?;
	let address = SocketAddress::from_str(&request.address)
		.map_err(|_| ldk_node::NodeError::InvalidSocketAddress)?;
	let user_channel_id = node.connect_open_channel(
		node_id,
		address,
		request.channel_amount_sats,
		request.push_to_counterparty_msat,
		None,
		request.announce_channel,
	)?;
	let response =
		OpenChannelResponse { user_channel_id: user_channel_id.0.to_be_bytes().to_vec() };
	Ok(response)
}

fn handle_close_channel(
	node: Arc<Node>, request: CloseChannelRequest,
) -> Result<CloseChannelResponse, ldk_node::NodeError> {
	let mut be_bytes = [0u8; 16];
	be_bytes.copy_from_slice(&request.user_channel_id);
	let user_channel_id = UserChannelId(u128::from_be_bytes(be_bytes));
	let counterparty_node_id = PublicKey::from_str(&request.counterparty_node_id)
		.map_err(|_| ldk_node::NodeError::InvalidNodeId)?;
	node.close_channel(&user_channel_id, counterparty_node_id)?;
	let response = CloseChannelResponse {};
	Ok(response)
}

fn handle_force_close_channel(
	node: Arc<Node>, request: ForceCloseChannelRequest,
) -> Result<ForceCloseChannelResponse, ldk_node::NodeError> {
	let mut be_bytes = [0u8; 16];
	be_bytes.copy_from_slice(&request.user_channel_id);
	let user_channel_id = UserChannelId(u128::from_be_bytes(be_bytes));
	let counterparty_node_id = PublicKey::from_str(&request.counterparty_node_id)
		.map_err(|_| ldk_node::NodeError::InvalidNodeId)?;
	node.force_close_channel(&user_channel_id, counterparty_node_id)?;
	let response = ForceCloseChannelResponse {};
	Ok(response)
}

fn to_payment_kind_proto(kind: &PaymentKind) -> protos::PaymentKind {
	match kind {
		ldk_node::payment::PaymentKind::Onchain => protos::PaymentKind {
			kind: Some(protos::payment_kind::Kind::Onchain(protos::Onchain {})),
		},
		ldk_node::payment::PaymentKind::Bolt11 { hash, preimage, secret } => protos::PaymentKind {
			kind: Some(protos::payment_kind::Kind::Bolt11(protos::Bolt11 {
				hash: hash.to_string(),
				preimage: preimage.map(|it| it.to_string()),
				secret: secret.map(|it| it.0.to_vec()),
			})),
		},
		ldk_node::payment::PaymentKind::Bolt11Jit { hash, preimage, secret, lsp_fee_limits } => {
			protos::PaymentKind {
				kind: Some(protos::payment_kind::Kind::Bolt11Jit(protos::Bolt11Jit {
					hash: hash.to_string(),
					preimage: preimage.map(|it| it.to_string()),
					secret: secret.map(|it| it.0.to_vec()),
					lsp_fee_limits: Some(protos::LspFeeLimits {
						max_total_opening_fee_msat: lsp_fee_limits.max_total_opening_fee_msat,
						max_proportional_opening_fee_ppm_msat: lsp_fee_limits
							.max_proportional_opening_fee_ppm_msat,
					}),
				})),
			}
		},
		ldk_node::payment::PaymentKind::Bolt12Offer { hash, preimage, secret, offer_id } => {
			protos::PaymentKind {
				kind: Some(protos::payment_kind::Kind::Bolt12offer(protos::Bolt12Offer {
					hash: hash.map(|it| it.to_string()),
					preimage: preimage.map(|it| it.to_string()),
					secret: secret.map(|it| it.0.to_vec()),
					offer_id: offer_id.0.to_vec(),
				})),
			}
		},
		ldk_node::payment::PaymentKind::Bolt12Refund { hash, preimage, secret } => {
			protos::PaymentKind {
				kind: Some(protos::payment_kind::Kind::Bolt12refund(protos::Bolt12Refund {
					hash: hash.map(|it| it.to_string()),
					preimage: preimage.map(|it| it.to_string()),
					secret: secret.map(|it| it.0.to_vec()),
				})),
			}
		},
		ldk_node::payment::PaymentKind::Spontaneous { hash, preimage } => protos::PaymentKind {
			kind: Some(protos::payment_kind::Kind::Spontaneous(protos::Spontaneous {
				hash: hash.to_string(),
				preimage: preimage.map(|it| it.to_string()),
			})),
		},
	}
}

fn to_payment_details_proto(payment: &PaymentDetails) -> protos::PaymentDetails {
	protos::PaymentDetails {
		id: Some(protos::PaymentId { data: payment.id.0.to_vec() }),
		kind: Some(to_payment_kind_proto(&payment.kind)),
		amount_msat: payment.amount_msat,
		direction: match payment.direction {
			PaymentDirection::Inbound => protos::PaymentDirection::Inbound.into(),
			PaymentDirection::Outbound => protos::PaymentDirection::Outbound.into(),
		},
		status: match payment.status {
			PaymentStatus::Pending => protos::PaymentStatus::Pending.into(),
			PaymentStatus::Succeeded => protos::PaymentStatus::Succeeded.into(),
			PaymentStatus::Failed => protos::PaymentStatus::Failed.into(),
		},
		latest_update_timestamp: payment.latest_update_timestamp,
	}
}
