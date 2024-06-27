use ldk_node::lightning::chain::BestBlock;
use ldk_node::payment::{PaymentDetails, PaymentDirection, PaymentKind, PaymentStatus};
use ldk_node::LightningBalance::{
	ClaimableAwaitingConfirmations, ClaimableOnChannelClose, ContentiousClaimable,
	CounterpartyRevokedOutputClaimable, MaybePreimageClaimableHTLC, MaybeTimeoutClaimableHTLC,
};
use ldk_node::Node;
use ldk_node::PendingSweepBalance::{
	AwaitingThresholdConfirmations, BroadcastAwaitingConfirmation, PendingBroadcast,
};
use prost::Message;

use core::future::Future;
use core::pin::Pin;
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::Service;
use hyper::{Request, Response};

use std::sync::Arc;

use protos::{
	lightning_balance, pending_sweep_balance, Channel, GetNodeStatusResponse, ListChannelsRequest,
	ListChannelsResponse, OnchainReceiveRequest, OnchainReceiveResponse, Outpoint,
};

const GET_NODE_STATUS_PATH: &str = "/getNodeStatus";
const ONCHAIN_RECEIVE: &str = "/onchain/receive";
const GET_NODE_BALANCES_PATH: &str = "/getNodeBalances";
const LIST_CHANNELS_PATH: &str = "/listChannels";
const PAYMENTS_HISTORY_PATH: &str = "/listPaymentsHistory";

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

	fn handle_get_payment_history_request(
		&self, _: Req,
	) -> <NodeService as Service<Request<Incoming>>>::Future {
		let payments = self.node.list_payments();
		let msg = protos::PaymentsHistoryResponse {
			payments: payments.iter().map(to_payment_details_proto).collect(),
		};
		make_response(msg.encode_to_vec())
	}

	fn default_response(&self) -> <NodeService as Service<Request<Incoming>>>::Future {
		make_response(b"UNKNOWN REQUEST".to_vec())
	}
}

async fn handle_onchain_receive(
	node: Arc<Node>, request: Req,
) -> Result<<NodeService as Service<Request<Incoming>>>::Response, hyper::Error> {
	// FIXME: Limit how much we read and add error checks
	let bytes = request.into_body().collect().await.unwrap().to_bytes();
	let _request = OnchainReceiveRequest::decode(bytes).unwrap();
	let response = OnchainReceiveResponse {
		address: node.onchain_payment().new_address().unwrap().to_string(),
	};
	Ok(Response::builder().body(Full::new(Bytes::from(response.encode_to_vec()))).unwrap())
}

async fn handle_get_balances_request(
	node: Arc<Node>, _: Req,
) -> Result<<NodeService as Service<Request<Incoming>>>::Response, hyper::Error> {
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
	let msg = protos::GetBalancesResponse {
		total_onchain_balance_sats: balance_details.total_onchain_balance_sats,
		spendable_onchain_balance_sats: balance_details.spendable_onchain_balance_sats,
		total_anchor_channels_reserve_sats: balance_details.total_anchor_channels_reserve_sats,
		total_lightning_balance_sats: balance_details.total_lightning_balance_sats,
		lightning_balances,
		pending_balances_from_channel_closures,
	};
	Ok(Response::builder().body(Full::new(Bytes::from(msg.encode_to_vec()))).unwrap())
}

async fn handle_list_channels_request(
	node: Arc<Node>, request: Req,
) -> Result<<NodeService as Service<Request<Incoming>>>::Response, hyper::Error> {
	let bytes = request.into_body().collect().await.unwrap().to_bytes();
	let _request = ListChannelsRequest::decode(bytes).unwrap();
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

	Ok(Response::builder().body(Full::new(Bytes::from(response.encode_to_vec()))).unwrap())
}

impl Service<Req> for NodeService {
	type Response = Response<Full<Bytes>>;
	type Error = hyper::Error;
	type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

	fn call(&self, req: Req) -> Self::Future {
		println!("processing request: {} {}", req.method(), req.uri().path());
		let node = Arc::clone(&self.node);
		match req.uri().path() {
			GET_NODE_STATUS_PATH => self.handle_get_node_status_request(req),
			GET_NODE_BALANCES_PATH => {
				Box::pin(async { handle_get_balances_request(node, req).await })
			},
			ONCHAIN_RECEIVE => Box::pin(async { handle_onchain_receive(node, req).await }),
			LIST_CHANNELS_PATH => Box::pin(async { handle_list_channels_request(node, req).await }),
			PAYMENTS_HISTORY_PATH => self.handle_get_payment_history_request(req),
			_ => self.default_response(),
		}
	}
}

fn make_response(bytes: Vec<u8>) -> <NodeService as Service<Request<Incoming>>>::Future {
	Box::pin(async { Ok(Response::builder().body(Full::new(bytes.into())).unwrap()) })
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
