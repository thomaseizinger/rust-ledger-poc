use bitcoin::hashes::Hash;
use bitcoin::secp256k1;
use bitcoin::util::bip32::{ChainCode, ChildNumber, DerivationPath, ExtendedPubKey, Fingerprint};
use bitcoin::{Network, PublicKey, XpubIdentifier};
use ledger::TransportNativeHID;
use ledger_apdu::map_apdu_error_description;
use ledger_apdu::APDUCommand;
use std::fmt;
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    // pretty_env_logger::try_init()?;

    let transport = TransportNativeHID::new()?;

    let (command, parse_response) = get_master_extended_public_key();
    let result = transport.exchange(&command)?;

    if result.retcode != 0x9000 {
        anyhow::bail!("{}", map_apdu_error_description(result.retcode))
    }
    let master_key = parse_response(result.data)?;

    let path = DerivationPath::from_str("m/44'/1'/0'")?;
    let (command, parse_response) = get_wallet_extended_public_key(&path);
    let result = transport.exchange(&command)?;

    if result.retcode != 0x9000 {
        anyhow::bail!("{}", map_apdu_error_description(result.retcode))
    }

    let wallet_key = parse_response(result.data)?;

    let descriptor = WalletDescriptor {
        master_key,
        wallet_path: path,
        wallet_key,
        chain: ChildNumber::from(0), // external
    };

    println!("{}", descriptor);

    Ok(())
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

pub fn get_master_extended_public_key() -> (
    APDUCommand,
    impl FnOnce(Vec<u8>) -> anyhow::Result<ExtendedPubKey>,
) {
    (
        APDUCommand {
            cla: 0xe0,
            ins: 0x40,
            p1: 0, // no verify
            p2: 2, // bech32
            data: path_to_be_bytes(&DerivationPath::from_str("m").unwrap()),
        },
        |data| {
            let (len, bytes) = data.split_first().unwrap();
            let (public_key, bytes) = bytes.split_at(*len as usize);
            let public_key = PublicKey {
                compressed: true,
                key: secp256k1::PublicKey::from_slice(public_key)?,
            };

            let (len, bytes) = bytes.split_first().unwrap();
            let (_, bytes) = bytes.split_at(*len as usize);
            let chain_code = ChainCode::from(bytes);

            Ok(ExtendedPubKey {
                network: Network::Testnet,
                depth: 0,
                parent_fingerprint: Default::default(),
                child_number: ChildNumber::from_normal_idx(0)?,
                public_key,
                chain_code,
            })
        },
    )
}

pub fn get_wallet_extended_public_key<'p>(
    path: &'p DerivationPath,
) -> (
    APDUCommand,
    impl FnOnce(Vec<u8>) -> anyhow::Result<ExtendedPubKey> + 'p,
) {
    (
        APDUCommand {
            cla: 0xe0,
            ins: 0x40,
            p1: 0, // no verify
            p2: 2, // bech32
            data: path_to_be_bytes(&path),
        },
        move |data| {
            let (len, bytes) = data.split_first().unwrap();
            let (public_key, bytes) = bytes.split_at(*len as usize);
            let public_key = PublicKey {
                compressed: true,
                key: secp256k1::PublicKey::from_slice(public_key)?,
            };

            let (len, bytes) = bytes.split_first().unwrap();
            let (_, bytes) = bytes.split_at(*len as usize);
            let chain_code = ChainCode::from(bytes);

            Ok(ExtendedPubKey {
                network: Network::Testnet,
                depth: path.as_ref().len() as u8,
                parent_fingerprint: Fingerprint::from(
                    &XpubIdentifier::hash(&public_key.to_bytes())[0..4],
                ),
                child_number: ChildNumber::from_normal_idx(0)?,
                public_key,
                chain_code,
            })
        },
    )
}

fn path_to_be_bytes(path: &DerivationPath) -> Vec<u8> {
    let child_numbers: &[ChildNumber] = path.as_ref();
    let p: Vec<u32> = child_numbers.iter().map(|&x| u32::from(x)).collect();
    let mut data: Vec<u8> = vec![child_numbers.len() as u8];
    for child_number in p {
        data.extend(&child_number.to_be_bytes());
    }
    data
}
