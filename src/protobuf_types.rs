#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNodeInfoRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetNodeInfoResponse {
    #[prost(string, tag = "1")]
    pub public_key: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub current_best_block: ::core::option::Option<BestBlock>,
    #[prost(uint64, optional, tag = "3")]
    pub latest_wallet_sync_timestamp: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "4")]
    pub latest_onchain_wallet_sync_timestamp: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "5")]
    pub latest_fee_rate_cache_update_timestamp: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "6")]
    pub latest_rgs_snapshot_timestamp: ::core::option::Option<u64>,
    #[prost(uint64, optional, tag = "7")]
    pub latest_node_announcement_broadcast_timestamp: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BestBlock {
    #[prost(string, tag = "1")]
    pub block_hash: ::prost::alloc::string::String,
    #[prost(uint32, tag = "2")]
    pub height: u32,
}
/// Retrieve a new on-chain/funding address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OnchainReceiveRequest {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OnchainRecevieResponse {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
/// Send an on-chain payment to the given address.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OnchainSendRequest {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub amount_sats: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OnchainSendResponse {
    #[prost(string, tag = "1")]
    pub txid: ::prost::alloc::string::String,
}
/// Return a BOLT11 invoice for the given amount, if specified.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt11ReceiveRequest {
    #[prost(string, tag = "1")]
    pub description: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub expiry_secs: u64,
    #[prost(uint64, optional, tag = "3")]
    pub amount_msat: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt11ReceiveResponse {
    #[prost(string, tag = "1")]
    pub invoice: ::prost::alloc::string::String,
}
/// Send a payment for a BOLT11 invoice.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt11SendRequest {
    #[prost(string, tag = "1")]
    pub invoice: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub amount_msat: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt11SendResponse {
    #[prost(message, optional, tag = "1")]
    pub payment_id: ::core::option::Option<PaymentId>,
}
/// Return a BOLT12 offer for the given amount, if specified.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt12ReceiveRequest {
    #[prost(string, tag = "1")]
    pub description: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub amount_msat: ::core::option::Option<u64>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt12ReceiveResponse {
    #[prost(string, tag = "1")]
    pub offer: ::prost::alloc::string::String,
}
/// Send a payment for a BOLT11 invoice.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt12SendRequest {
    #[prost(string, tag = "1")]
    pub offer: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "2")]
    pub amount_msat: ::core::option::Option<u64>,
    #[prost(string, optional, tag = "3")]
    pub payer_note: ::core::option::Option<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Bolt12SendResponse {
    #[prost(message, optional, tag = "1")]
    pub payment_id: ::core::option::Option<PaymentId>,
}
/// An identifier for making a payment.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PaymentId {
    #[prost(bytes = "vec", tag = "1")]
    pub data: ::prost::alloc::vec::Vec<u8>,
}
