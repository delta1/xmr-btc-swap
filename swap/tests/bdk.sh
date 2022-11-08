#!/bin/bash

set -euxo pipefail

VERSION=0.11.1

echo "download swap $VERSION"
curl -L "https://github.com/comit-network/xmr-btc-swap/releases/download/${VERSION}/swap_${VERSION}_Linux_x86_64.tar" | tar xv
ls -alht
ls -alht swap
file swap

echo "create mainnet wallet with $VERSION"
./swap --version || exit 1
./swap --data-base-dir . --debug balance || exit 1
echo "check mainnet wallet with this version"
./target/debug/swap --version || exit 1
./target/debug/swap --data-base-dir . --debug balance || exit 1

echo "create testnet wallet with $VERSION"
./swap --testnet --data-base-dir . --debug balance || exit 1
echo "check testnet wallet with this version"
./target/debug/swap --testnet --data-base-dir . --debug balance || exit 1

exit 0
