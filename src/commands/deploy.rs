//! Deploy command implementation

use crate::compiled::CompiledOutput;
use crate::error::SprayError;
use crate::file_loader;
use colored::Colorize;
use musk::client::NodeClient;
use musk::Network;
use std::path::{Path, PathBuf};

/// Execute the deploy command
///
/// # Errors
///
/// Returns an error if deployment fails or file operations fail.
#[allow(clippy::too_many_arguments)]
pub fn deploy_command(
    file: &Path,
    args: Option<PathBuf>,
    amount: Option<u64>,
    asset: Option<String>,
    network: Network,
    config: Option<PathBuf>,
) -> Result<(), SprayError> {
    println!("{}", "Deploying Simplicity contract...".cyan().bold());
    println!();

    // Create network backend
    println!("{} {network}", "Network:".dimmed());
    let backend = crate::network::create_backend(network, config)?;

    // Detect file type and compile if needed
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| SprayError::FileFormatError("No file extension found".into()))?;

    let compiled = match ext {
        "simf" => {
            // Compile from source
            println!("{} {}", "Compiling from source:".dimmed(), file.display());
            let source = std::fs::read_to_string(file)?;
            let contract = musk::Contract::from_source(&source)?;

            let arguments = if let Some(args_path) = args {
                println!("{} {}", "Loading arguments from:".dimmed(), args_path.display());
                file_loader::load_arguments(&args_path)?
            } else {
                musk::Arguments::default()
            };

            println!("{}", "Compiling...".dimmed());
            contract.instantiate(arguments)?
        }
        "json" => {
            // Load pre-compiled
            println!("{} {}", "Loading pre-compiled contract:".dimmed(), file.display());
            let json_str = std::fs::read_to_string(file)?;
            let output: CompiledOutput = serde_json::from_str(&json_str)?;

            // For now, we need to recompile from source if it's available
            if let Some(source) = output.source {
                let contract = musk::Contract::from_source(&source)?;
                let arguments = if let Some(args_path) = args {
                    file_loader::load_arguments(&args_path)?
                } else {
                    musk::Arguments::default()
                };
                contract.instantiate(arguments)?
            } else {
                return Err(SprayError::FileFormatError(
                    "Pre-compiled JSON must include source field for deployment".into(),
                ));
            }
        }
        _ => {
            return Err(SprayError::FileFormatError(format!(
                "Unsupported file extension: {ext} (expected .simf or .json)"
            )));
        }
    };

    // Get contract address
    let address = compiled.address(backend.address_params());
    println!();
    println!("{}", "Contract address:".bold());
    println!("  {address}");
    println!();

    // Determine amount (default 1 BTC)
    let amount_sats = amount.unwrap_or(100_000_000);
    println!("{} {} sat", "Sending amount:".dimmed(), amount_sats);

    // Send funds to contract address
    println!("{}", "Creating funding transaction...".dimmed());
    let txid = backend
        .send_to_address(&address, amount_sats)
        .map_err(|e| SprayError::RpcError(e.to_string()))?;

    // Get the transaction to find the vout
    let tx = backend
        .get_transaction(&txid)
        .map_err(|e| SprayError::RpcError(e.to_string()))?;

    // Find the output index
    let script_pubkey = address.script_pubkey();
    let vout = tx
        .output
        .iter()
        .position(|output| output.script_pubkey == script_pubkey)
        .ok_or_else(|| SprayError::TestError("Could not find output in transaction".into()))?;

    println!();
    println!("{}", "✓ Deployment successful!".green().bold());
    println!();
    println!("{}", "Funding details:".bold());
    println!("  {} {txid}", "Txid:".bold());
    println!("  {} {vout}", "Vout:".bold());
    println!("  {} {amount_sats} sat", "Amount:".bold());
    
    if let Some(asset_id) = asset {
        println!("  {} {asset_id}", "Asset:".bold());
    }

    println!();
    println!("{}", "To spend from this UTXO:".dimmed());
    println!("  spray redeem {txid}:{vout} <witness.json>");

    Ok(())
}

