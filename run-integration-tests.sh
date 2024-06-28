#!/usr/bin/env bash
set +eux pipefail
# Usage: After running docker compose up, execute this script to check that each 
# cli command succeeds. The CLI will make requests to the server.
pushd cli
cargo run -- --base-url localhost:3000 node-id
cargo run -- --base-url localhost:3000 node-status
cargo run -- --base-url localhost:3000 new-address
# Currently fails due to insufficient funds
cargo run -- --base-url localhost:3000 send-onchain bcrt1qcupfx8l06ads2ej9n3h7gcvpc3lhnm8708xawy 1000
cargo run -- --base-url localhost:3000 bolt11-receive "description" 1000 1000
cargo run -- --base-url localhost:3000 node-balances
cargo run -- --base-url localhost:3000 payments-history
# Payment id is invalid, need to get a legit one from bolt11-send
cargo run -- --base-url localhost:3000 payment-details -p bfb1737ad6a575e44be343d73da4655e36ddea8c1a4479522e22f054a80705be
cargo run -- --base-url localhost:3000 list-channels
# Currently fails due to insufficient funds
cargo run -- --base-url localhost:3000 open-channel --node-id 027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190 --address localhost:3042 --channel-amount-sats 1000000 --announce-channel
cargo run -- --base-url localhost:3000 close-channel --user-channel-id 1234567890123456 --counterparty-node-id 027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190
cargo run -- --base-url localhost:3000 force-close-channel --user-channel-id 1234567890123456 --counterparty-node-id 027100442c3b79f606f80f322d98d499eefcb060599efc5d4ecb00209c2cb54190

popd
