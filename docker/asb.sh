#! /bin/bash
set -o errexit
set -o pipefail
# set -o nounset
# set -o xtrace

monero-wallet-rpc --stagenet --daemon-host stagenet.community.rino.io --rpc-bind-port 18083 --disable-rpc-login --wallet-dir /home/asb/data &

exec "$@"
