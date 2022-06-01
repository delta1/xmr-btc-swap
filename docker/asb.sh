#! /bin/bash
set -o errexit
set -o pipefail
# set -o nounset
# set -o xtrace

if [ -n "$DATA_DIR" ]; then
    ASB_DATA_DIR=$DATA_DIR
else
    ASB_DATA_DIR="/home/asb/data"
fi

if [ -n "$NETWORK_LISTEN" ]; then
    ASB_NETWORK_LISTEN=$NETWORK_LISTEN
else
    ASB_NETWORK_LISTEN=("/ip4/0.0.0.0/tcp/9939" "/ip4/0.0.0.0/tcp/9940/ws")
fi

if [ -n "$NETWORK_RENDEZVOUS_POINT" ]; then
    ASB_NETWORK_RENDEZVOUS_POINT=$NETWORK_RENDEZVOUS_POINT
else
    ASB_NETWORK_RENDEZVOUS_POINT=
fi

if [ -n "$NETWORK_EXTERNAL_ADDRESSES" ]; then
    ASB_NETWORK_EXTERNAL_ADDRESSES=$NETWORK_EXTERNAL_ADDRESSES
else
    ASB_NETWORK_EXTERNAL_ADDRESSES=()
fi

if [ -n "$BITCOIN_ELECTRUM_RPC_URL" ]; then
    ASB_BITCOIN_ELECTRUM_RPC_URL=$BITCOIN_ELECTRUM_RPC_URL
else
    ASB_BITCOIN_ELECTRUM_RPC_URL="ssl://blockstream.info:700"
fi

if [ -n "$BITCOIN_TARGET_BLOCK" ]; then
    ASB_BITCOIN_TARGET_BLOCK=$BITCOIN_TARGET_BLOCK
else
    ASB_BITCOIN_TARGET_BLOCK=3
fi

if [ -n "$BITCOIN_FINALITY_CONFIRMATIONS" ]; then
    ASB_BITCOIN_FINALITY_CONFIRMATIONS=$BITCOIN_FINALITY_CONFIRMATIONS
else
    ASB_BITCOIN_FINALITY_CONFIRMATIONS=
fi

if [ -n "$BITCOIN_NETWORK" ]; then
    ASB_BITCOIN_NETWORK=$BITCOIN_NETWORK
else
    ASB_BITCOIN_NETWORK=Mainnet
fi

if [ -n "$MONERO_WALLET_RPC_URL" ]; then
    ASB_MONERO_WALLET_RPC_URL=$MONERO_WALLET_RPC_URL
else
    ASB_MONERO_WALLET_RPC_URL="http://127.0.0.1:18083/json_rpc"
fi

if [ -n "$MONERO_FINALITY_CONFIRMATIONS" ]; then
    ASB_MONERO_FINALITY_CONFIRMATIONS=$MONERO_FINALITY_CONFIRMATIONS
else
    ASB_MONERO_FINALITY_CONFIRMATIONS=
fi

if [ -n "$MONERO_NETWORK" ]; then
    ASB_MONERO_NETWORK=$MONERO_NETWORK
else
    ASB_MONERO_NETWORK=Mainnet
fi

if [ -n "$TOR_CONTROL_PORT" ]; then
    ASB_TOR_CONTROL_PORT=$TOR_CONTROL_PORT
else
    ASB_TOR_CONTROL_PORT=9051
fi

if [ -n "$TOR_SOCKS5_PORT" ]; then
    ASB_TOR_SOCKS5_PORT=$TOR_SOCKS5_PORT
else
    ASB_TOR_SOCKS5_PORT=9050
fi

if [ -n "$MAKER_MIN_BUY_BTC" ]; then
    ASB_MAKER_MIN_BUY_BTC=$MAKER_MIN_BUY_BTC
else
    ASB_MAKER_MIN_BUY_BTC=0.002
fi

if [ -n "$MAKER_MAX_BUY_BTC" ]; then
    ASB_MAKER_MAX_BUY_BTC=$MAKER_MAX_BUY_BTC
else
    ASB_MAKER_MAX_BUY_BTC=0.02
fi

if [ -n "$MAKER_ASK_SPREAD" ]; then
    ASB_MAKER_ASK_SPREAD=$MAKER_ASK_SPREAD
else
    ASB_MAKER_ASK_SPREAD=0.02
fi

if [ -n "$MAKER_PRICE_TICKER_WS_URL" ]; then
    ASB_MAKER_PRICE_TICKER_WS_URL=$MAKER_PRICE_TICKER_WS_URL
else
    ASB_MAKER_PRICE_TICKER_WS_URL="wss://ws.kraken.com/"
fi

if [ ${#ASB_NETWORK_EXTERNAL_ADDRESSES[@]} -eq 0 ]; then
    EXTERNAL_ADDRESSES=$(jo -a -n "")
else
    EXTERNAL_ADDRESSES=$(jo -a "${ASB_NETWORK_EXTERNAL_ADDRESSES[@]}")
fi

NETWORK=$(jo listen="$(jo -a "${ASB_NETWORK_LISTEN[@]}")" rendezvous_point="$ASB_NETWORK_RENDEZVOUS_POINT" external_addresses="$EXTERNAL_ADDRESSES")

BITCOIN=$(jo electrum_rpc_url="$ASB_BITCOIN_ELECTRUM_RPC_URL" target_block="$ASB_BITCOIN_TARGET_BLOCK" finality_confirmations="$ASB_BITCOIN_FINALITY_CONFIRMATIONS" network="$ASB_BITCOIN_NETWORK")

MONERO=$(jo wallet_rpc_url="$ASB_MONERO_WALLET_RPC_URL" finality_confirmations="$ASB_MONERO_FINALITY_CONFIRMATIONS" network="$ASB_MONERO_NETWORK")

TOR=$(jo control_port="$ASB_TOR_CONTROL_PORT" socks5_port="$ASB_TOR_SOCKS5_PORT")

MAKER=$(jo min_buy_btc="$ASB_MAKER_MIN_BUY_BTC" max_buy_btc="$ASB_MAKER_MAX_BUY_BTC" ask_spread="$ASB_MAKER_ASK_SPREAD" price_ticker_ws_url="$ASB_MAKER_PRICE_TICKER_WS_URL")

jo -p data[dir]="$ASB_DATA_DIR" network="$NETWORK" bitcoin="$BITCOIN" monero="$MONERO" tor="$TOR" maker="$MAKER" >/etc/asb/config.json

monero-wallet-rpc --stagenet --daemon-host stagenet.community.rino.io --rpc-bind-port 18083 --disable-rpc-login --wallet-dir /home/asb/data &

exec "$@"
