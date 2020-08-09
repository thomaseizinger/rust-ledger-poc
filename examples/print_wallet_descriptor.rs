use bitcoin::util::bip32::{ChainCode, ChildNumber, DerivationPath, ExtendedPubKey, Fingerprint};
use bitcoin::{Network, PublicKey};
use coins_bip32::model::HasPubkey;
use coins_bip32::{DerivedXPub, XKey};
use std::env::args;
use std::fmt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let chain = args()
        .nth(1)
        .unwrap_or_else(|| String::from("0"))
        .parse::<u32>()?
        .into();

    let app = bitcoins_ledger::LedgerBTC::init().await?;

    let path = "m/44'/1'/0'";
    let master = app.get_master_xpub().await?;
    let wallet = app.get_xpub(&path.parse()?).await?;

    println!(
        "{}",
        WalletDescriptor {
            master_key: into_extended_pub_key(master)?,
            wallet_path: path.parse()?,
            wallet_key: into_extended_pub_key(wallet)?,
            chain,
        }
    );

    Ok(())
}

fn into_extended_pub_key(master: DerivedXPub) -> anyhow::Result<ExtendedPubKey> {
    Ok(ExtendedPubKey {
        network: Network::Testnet,
        depth: master.depth(),
        parent_fingerprint: Fingerprint::from(master.parent().0.as_ref()),
        child_number: ChildNumber::from(master.index()),
        public_key: PublicKey::from_slice(&master.pubkey_bytes())?,
        chain_code: ChainCode::from(master.chain_code().0.as_ref()),
    })
}

struct WalletDescriptor {
    master_key: ExtendedPubKey,
    wallet_path: DerivationPath,
    wallet_key: ExtendedPubKey,
    chain: ChildNumber,
}

impl fmt::Display for WalletDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wpkh([{}", self.master_key.fingerprint())?;
        for child in self.wallet_path.as_ref() {
            write!(f, "/{}", child)?;
        }
        write!(f, "]{}/{}/*)", self.wallet_key, self.chain)
    }
}
