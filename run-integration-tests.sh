#!/usr/bin/env bash
set +eux pipefail
# Usage: After running docker compose up, execute this script to check that each 
# cli command succeeds. The CLI will make requests to the server.
pushd cli
cargo run -- --base-url localhost:3000 node-status
cargo run -- --base-url localhost:3000 node-balances
cargo run -- --base-url localhost:3000 payments-history

popd
