#!/bin/bash

set -euxo pipefail

VERSION=0.11.1

mkdir compat
# stat ./target/debug/swap || exit 1
# stat ./target/debug/asb || exit 1
# cp ./target/debug/swap compat/swap-current
# cp ./target/debug/asb compat/asb-current
pushd compat

echo "download monero 0.18.3.1"
curl -L "https://downloads.getmonero.org/cli/monero-linux-x64-v0.18.3.1.tar.bz2" | bunzip2 | tar xv
echo "download swap $VERSION"
curl -L "https://github.com/comit-network/xmr-btc-swap/releases/download/${VERSION}/swap_${VERSION}_Linux_x86_64.tar" | tar xv
echo "download asb $VERSION"
curl -L "https://github.com/comit-network/xmr-btc-swap/releases/download/${VERSION}/asb_${VERSION}_Linux_x86_64.tar" | tar xv
echo "download rendezvous-server 0.2.0"
curl -L "https://github.com/comit-network/rendezvous-server/releases/download/0.2.0/rendezvous-server_0.2.0_Linux_x86_64.tar" | tar xv

ls -alht
pwd

exit 1

exit 0
