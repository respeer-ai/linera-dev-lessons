#!/bin/bash

cargo build --release --target wasm32-unknown-unknown

rm ~/.config/linera -rf
linera wallet init --faucet https://faucet.testnet-conway.linera.net

linera wallet request-chain --faucet https://faucet.testnet-conway.linera.net
linera wallet request-chain --faucet https://faucet.testnet-conway.linera.net

MODULE_ID=$(linera publish-module ./target/wasm32-unknown-unknown/release/fungible_token_{contract,service}.wasm)
linera create-application $MODULE_ID \
    --json-argument '{"total_supply": "21000000.", "name": "Linera Test Fungible Token", "symbol": "LTFT", "mint_ratio": "10.2", "decimals": 6}'

linera service --port 8080
