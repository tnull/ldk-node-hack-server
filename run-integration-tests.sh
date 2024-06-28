#!/usr/bin/env bash
set +eux pipefail
# Usage: After running docker compose up, execute this script to check that each 
# cli command succeeds. The CLI will make requests to the server.
pushd cli
cargo run -- --base-url localhost:3000 node-id
cargo run -- --base-url localhost:3000 node-status
cargo run -- --base-url localhost:3000 new-address
cargo run -- --base-url localhost:3000 send-onchain addy 1000
cargo run -- --base-url localhost:3000 bolt11-receive "description" 1000 1000
cargo run -- --base-url localhost:3000 bolt11-send invoice 1000
cargo run -- --base-url localhost:3000 bolt12-receive "description" 1000
cargo run -- --base-url localhost:3000 bolt11-send offer 1000
cargo run -- --base-url localhost:3000 node-balances
cargo run -- --base-url localhost:3000 payments-history
cargo run -- --base-url localhost:3000 payment-details -p 12345678901234567890123456789012
cargo run -- --base-url localhost:3000 list-channels
cargo run -- --base-url localhost:3000 open-channel --node-id 027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190 --address localhost:3042 --channel-amount-sats 1000000 --announce-channel
cargo run -- --base-url localhost:3000 close-channel --user-channel-id 1234567890123456 --counterparty-node-id 027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190
cargo run -- --base-url localhost:3000 force-close-channel --user-channel-id 1234567890123456 --counterparty-node-id 027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190

popd
