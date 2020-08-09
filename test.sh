set -e

docker rm -f bitcoind-regtest > /dev/null
docker run -d -p 18443:18443 --name bitcoind-regtest coblox/bitcoin-core:0.20.0 -regtest -txindex=1 -debug -rpcallowip=172.0.0.0/8 -rpcbind=0.0.0.0 -minrelaytxfee=0> /dev/null

sleep 2

EXTERNAL_WALLET_DESCRIPTOR=$(cargo run --example print_wallet_descriptor 0);
INTERNAL_WALLET_DESCRIPTOR=$(cargo run --example print_wallet_descriptor 1);

echo "External wallet descriptor: $EXTERNAL_WALLET_DESCRIPTOR"
echo "Internal wallet descriptor: $INTERNAL_WALLET_DESCRIPTOR"

WALLET_NAME="nano-ledger-s-pink"

echo "Importing into wallet $WALLET_NAME"
#
#sleep 2
#
#node ./import_xpub_into_bitcoind.js $WALLET_NAME "$EXTERNAL_WALLET_DESCRIPTOR" "$INTERNAL_WALLET_DESCRIPTOR"
#
#ADDRESS=$(node ./get_new_address.js)
#
#echo "Funding address $ADDRESS"
#
#docker exec bitcoind-regtest bitcoin-cli -regtest generatetoaddress 101 "$ADDRESS" > /dev/null
#
#BALANCE=$(docker exec bitcoind-regtest bitcoin-cli -regtest -rpcwallet=$WALLET_NAME getbalance "*" 0 true)
#
#echo "Balance: $BALANCE"
#
#test "$BALANCE" != "0.00000000"
#
#RAW_TX=$(node ./create_spending_tx.js $WALLET_NAME)
#
#echo "Raw spending tx: $RAW_TX"
#
#docker exec bitcoind-regtest bitcoin-cli -regtest sendrawtransaction "$RAW_TX"