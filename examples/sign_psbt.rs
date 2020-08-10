use anyhow::Context;
use bitcoin::hashes::hex::ToHex;
use bitcoin::hashes::Hash;
use bitcoin::util::psbt::PartiallySignedTransaction;
use bitcoins::prelude::*;
use bitcoins_ledger::SigningInfo;
use coins_bip32::path::KeyDerivation;
use coins_bip32::KeyFingerprint;
use std::env::args;
use std::io::Cursor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = args().skip(1);

    let psbt = args.next().unwrap();
    let psbt_bytes = base64::decode(psbt)?;
    let mut psbt = bitcoin::consensus::deserialize::<PartiallySignedTransaction>(&psbt_bytes)
        .context("failed to deserialize psbt")?;

    let signing_info = psbt
        .inputs
        .iter()
        .zip(psbt.global.unsigned_tx.input.iter())
        .enumerate()
        .map(|(index, (input, txin))| {
            let out = input
                .witness_utxo
                .as_ref()
                .ok_or(SpendingFromNonWitnessTx)?;
            let (_, (fingerprint, derivation_path)) =
                input.hd_keypaths.iter().next().ok_or(NoDerivationPath)?;

            Ok(SigningInfo {
                input_idx: index,
                prevout: UTXO::new(
                    BitcoinOutpoint {
                        txid: TXID::from(txin.previous_output.txid.into_inner()),
                        idx: txin.previous_output.vout,
                    },
                    out.value,
                    ScriptPubkey::new(out.script_pubkey.to_bytes()),
                    SpendScript::None,
                ),
                deriv: Some(KeyDerivation {
                    root: KeyFingerprint::from(fingerprint.into_bytes()),
                    path: derivation_path.to_string().parse()?,
                }),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let tx = psbt.clone().extract_tx();
    let mut tx_bytes = Cursor::new(bitcoin::consensus::serialize(&tx));
    let witness_tx = WitnessTx::from_legacy(
        LegacyTx::read_from(&mut tx_bytes, 0).context("failed to create WitnessTx")?,
    );

    let app = bitcoins_ledger::LedgerBTC::init().await?;
    let signatures = app
        .get_tx_signatures(&witness_tx, &signing_info)
        .await
        .context("failed to get signatures from device")?;

    for sig in signatures {
        let input = psbt.inputs.get_mut(sig.input_idx).unwrap();
        let signature = sig.sig.serialize_der().to_vec(); // TODO: is der-encoding correct here?
        input
            .partial_sigs
            .insert(input.hd_keypaths.keys().next().cloned().unwrap(), signature);
    }

    miniscript::psbt::finalize(&mut psbt)?;

    let hex = bitcoin::consensus::serialize(&psbt.extract_tx()).to_hex();

    println!("{}", hex);

    Ok(())
}

#[derive(thiserror::Error, Debug)]
#[error("attempted to spend a non-witness input")]
struct SpendingFromNonWitnessTx;

#[derive(thiserror::Error, Debug)]
#[error("no derivation path provided for key")]
struct NoDerivationPath;
