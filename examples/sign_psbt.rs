use bitcoins::enc::TestnetEncoder;
use bitcoins::prelude::ByteFormat;
use bitcoins_ledger::LedgerBTC;
use bitcoins_psbt::roles::PSTSigner;
use bitcoins_psbt::{PSBT, PST};
use std::env::args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let psbt = args().nth(1).unwrap();

    let mut psbt =
        PSBT::<TestnetEncoder, coins_bip32::enc::TestnetEncoder>::deserialize_base64(&psbt)?;

    let app = LedgerBTC::init().await?;
    app.sign(&mut psbt)?;

    let tx_hex = hex::encode(psbt.tx_bytes()?);

    println!("{}", tx_hex);

    Ok(())
}
