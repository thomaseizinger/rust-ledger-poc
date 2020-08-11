set -e

docker rm -f bitcoind-regtest > /dev/null
docker run -d -p 18443:18443 --name bitcoind-regtest coblox/bitcoin-core:0.20.0 -regtest -txindex=1 -debug -rpcallowip=172.0.0.0/8 -rpcbind=0.0.0.0 -minrelaytxfee=0> /dev/null

sleep 2

EXTERNAL_WALLET_DESCRIPTOR=$(cargo run --example print_wallet_descriptor 0);
INTERNAL_WALLET_DESCRIPTOR=$(cargo run --example print_wallet_descriptor 1);

echo "External wallet descriptor: $EXTERNAL_WALLET_DESCRIPTOR"
echo "Internal wallet descriptor: $INTERNAL_WALLET_DESCRIPTOR"

WALLET_NAME="nano-ledger-s"

echo "Importing into wallet $WALLET_NAME"

# Create wallet
docker exec bitcoind-regtest bitcoin-cli -regtest createwallet $WALLET_NAME true true > /dev/null

sleep 2

EXTERNAL_WALLET_DESCRIPTOR=$(docker exec bitcoind-regtest bitcoin-cli -regtest getdescriptorinfo "$EXTERNAL_WALLET_DESCRIPTOR" | jq -r '.descriptor')
INTERNAL_WALLET_DESCRIPTOR=$(docker exec bitcoind-regtest bitcoin-cli -regtest getdescriptorinfo "$INTERNAL_WALLET_DESCRIPTOR" | jq -r '.descriptor')

IMPORT_REQUEST_EXTERNAL=$(jq --arg DESC $EXTERNAL_WALLET_DESCRIPTOR '. + {desc: $DESC, internal: false}' import_multi_request.json)
IMPORT_REQUEST_INTERNAL=$(jq --arg DESC $INTERNAL_WALLET_DESCRIPTOR '. + {desc: $DESC, internal: true}' import_multi_request.json)

docker exec bitcoind-regtest bitcoin-cli -regtest -rpcwallet=$WALLET_NAME importmulti "[$IMPORT_REQUEST_EXTERNAL, $IMPORT_REQUEST_INTERNAL]" "{\"rescan\": true}" > /dev/null

ADDRESS=$(cargo run --example get_funding_address)

echo "Funding address $ADDRESS"

docker exec bitcoind-regtest bitcoin-cli -regtest generatetoaddress 101 "$ADDRESS" > /dev/null

BALANCE=$(docker exec bitcoind-regtest bitcoin-cli -regtest -rpcwallet=$WALLET_NAME getbalance "*" 0 true)

echo "Balance: $BALANCE"

test "$BALANCE" != "0.00000000"

PSBT=$(docker exec bitcoind-regtest bitcoin-cli -regtest -rpcwallet=$WALLET_NAME walletcreatefundedpsbt '[]' "[{\"$ADDRESS\": \"1\"}]" null "{\"feeRate\": 0}" | jq -r .psbt)

RAW_TX=$(cargo run --example sign_psbt $PSBT)

echo "Raw spending tx: $RAW_TX"

docker exec bitcoind-regtest bitcoin-cli -regtest sendrawtransaction "$RAW_TX"
