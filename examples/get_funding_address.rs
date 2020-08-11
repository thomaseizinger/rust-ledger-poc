use coins_bip32::model::HasPubkey;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = bitcoins_ledger::LedgerBTC::init().await?;

    let path = "m/84'/1'/0'/0/0";
    let wallet = app.get_xpub(&path.parse()?).await?;

    println!(
        "{}",
        bitcoin::Address::p2wpkh(
            &bitcoin::PublicKey::from_slice(&wallet.pubkey_bytes())?,
            bitcoin::Network::Regtest
        )
    );

    Ok(())
}
