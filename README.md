# rust-ledger-poc

A proof-of-concept for working with a Ledger Nano S from Rust.

Run the `test.sh` shell script to give it a go.
It expects your Nano Ledger S to be plugged in with the Bitcoin app open.

It will start a bitcoind node, create a watch-only wallet from the device's extended public key, fund it with some bitcoin and try and sign a transaction that spends to an arbitrary address using the Ledger and a PSBT created by the watch-only wallet.

Unfortunately, there is still an issue with the Ledger communication that I believe is an upstream bug.
