#!/bin/bash

VERSION=0.11.1

echo "download swap $VERSION"
curl -L "https://github.com/comit-network/xmr-btc-swap/releases/download/${VERSION}/swap_${VERSION}_Linux_x86_64.tar" | tar xv

echo "create mainnet wallet with $VERSION"
./swap --version
./swap --data-base-dir . --debug balance
echo "check mainnet wallet with this version"
./target/debug/swap --version
./target/debug/swap --data-base-dir . --debug balance

echo "create testnet wallet with $VERSION"
./swap --testnet --data-base-dir . --debug balance
echo "check testnet wallet with this version"
./target/debug/swap --testnet --data-base-dir . --debug balance
