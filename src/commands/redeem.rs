//! Redeem command implementation

use crate::compiled::CompiledOutput;
use crate::error::SprayError;
use crate::file_loader;
use colored::Colorize;
use musk::client::{NodeClient, Utxo};
use musk::elements::{confidential, encode::serialize_hex, LockTime, Sequence};
use musk::{Network, SpendBuilder};
use std::path::{Path, PathBuf};

/// Parse a UTXO reference in the format "txid:vout"
///
/// # Errors
///
/// Returns an error if the format is invalid.
fn parse_utxo_ref(s: &str) -> Result<(musk::Txid, u32), SprayError> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(SprayError::InvalidUtxoRef(format!(
            "Expected format 'txid:vout', got: {s}"
        )));
    }

    let txid = parts[0]
        .parse()
        .map_err(|e| SprayError::InvalidUtxoRef(format!("Invalid txid: {e}")))?;
    let vout = parts[1]
        .parse()
        .map_err(|e| SprayError::InvalidUtxoRef(format!("Invalid vout: {e}")))?;

    Ok((txid, vout))
}

/// Execute the redeem command
///
/// # Errors
///
/// Returns an error if redemption fails or file operations fail.
#[allow(clippy::too_many_arguments)]
pub fn redeem_command(
    utxo_ref: &str,
    witness_file: &Path,
    compiled_file: Option<PathBuf>,
    dest: Option<String>,
    fee: Option<u64>,
    network: Network,
    config: Option<PathBuf>,
) -> Result<(), SprayError> {
    println!("{}", "Redeeming from Simplicity contract...".cyan().bold());
    println!();

    // Parse UTXO reference
    let (txid, vout) = parse_utxo_ref(utxo_ref)?;
    println!("{} {txid}:{vout}", "UTXO:".dimmed());

    // Create network backend
    println!("{} {network}", "Network:".dimmed());
    let mut backend = crate::network::create_backend(network, config)?;

    // Get the transaction to find the UTXO
    println!("{}", "Fetching UTXO...".dimmed());
    let tx = backend
        .get_transaction(&txid)
        .map_err(|e| SprayError::RpcError(e.to_string()))?;

    let output = tx
        .output
        .get(vout as usize)
        .ok_or_else(|| SprayError::InvalidUtxoRef(format!("Vout {vout} not found in transaction")))?;

    // Extract amount and asset
    let confidential::Value::Explicit(amount) = output.value else {
        return Err(SprayError::TestError("Non-explicit value in UTXO".into()));
    };

    let confidential::Asset::Explicit(asset) = output.asset else {
        return Err(SprayError::TestError("Non-explicit asset in UTXO".into()));
    };

    println!("  {} {} sat", "Amount:".bold(), amount);
    println!("  {} {asset}", "Asset:".bold());

    // Load compiled contract
    let compiled_file = compiled_file.ok_or_else(|| {
        SprayError::FileFormatError("--compiled <file> is required for redeem command".into())
    })?;

    println!();
    println!("{} {}", "Loading contract from:".dimmed(), compiled_file.display());
    let json_str = std::fs::read_to_string(&compiled_file)?;
    let output_data: CompiledOutput = serde_json::from_str(&json_str)?;

    let source = output_data.source.ok_or_else(|| {
        SprayError::FileFormatError("Compiled contract must include source field".into())
    })?;

    let contract = musk::Contract::from_source(&source)?;
    let compiled = contract.instantiate(musk::Arguments::default())?;

    // Load witness
    println!("{} {}", "Loading witness from:".dimmed(), witness_file.display());
    let witness_values = file_loader::load_witness(witness_file)?;

    // Build UTXO struct
    let utxo = Utxo {
        txid,
        vout,
        amount,
        script_pubkey: output.script_pubkey.clone(),
        asset: output.asset,
    };

    // Get genesis hash
    let genesis_hash = backend.genesis_hash()?;

    // Determine destination
    let destination = if let Some(dest_str) = dest {
        dest_str
            .parse()
            .map_err(|e| SprayError::ParseError(format!("Invalid destination address: {e}")))?
    } else {
        backend
            .get_new_address()
            .map_err(|e| SprayError::RpcError(e.to_string()))?
    };

    // Determine fee (default 3000 sat)
    let fee_amount = fee.unwrap_or(3_000);
    let output_amount = amount
        .checked_sub(fee_amount)
        .ok_or_else(|| SprayError::TestError("Insufficient funds for fee".into()))?;

    println!();
    println!("{}", "Building spending transaction...".dimmed());
    println!("  {} {}", "Destination:".bold(), destination);
    println!("  {} {} sat", "Output amount:".bold(), output_amount);
    println!("  {} {} sat", "Fee:".bold(), fee_amount);

    // Build the spend
    let mut builder = SpendBuilder::new(compiled, utxo)
        .genesis_hash(genesis_hash)
        .lock_time(LockTime::ZERO)
        .sequence(Sequence::MAX);

    builder.add_output_simple(destination.script_pubkey(), output_amount, asset);
    builder.add_fee(fee_amount, asset);

    // Compute sighash
    let sighash = builder
        .sighash_all()
        .map_err(SprayError::SpendError)?;

    println!("  {} {}", "Sighash:".dimmed(), hex::encode(&sighash));

    // Finalize with witness
    println!("{}", "Finalizing transaction...".dimmed());
    let tx = builder
        .finalize(witness_values)
        .map_err(SprayError::SpendError)?;

    // Broadcast
    println!("{}", "Broadcasting transaction...".dimmed());
    let spend_txid = backend
        .broadcast(&tx)
        .map_err(|e| SprayError::RpcError(e.to_string()))?;

    println!();
    println!("{}", "✓ Redemption successful!".green().bold());
    println!();
    println!("{}", "Transaction details:".bold());
    println!("  {} {spend_txid}", "Txid:".bold());
    
    println!();
    println!("{}", "Raw transaction (hex):".dimmed());
    println!("{}", serialize_hex(&tx));

    Ok(())
}

// Add hex module
#[doc(hidden)]
mod hex {
    use std::fmt::Write;
    
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        })
    }
}

