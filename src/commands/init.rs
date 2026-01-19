//! Init command implementation

use crate::error::SprayError;
use colored::Colorize;
use std::fs;
use std::path::Path;

/// Default musk.conf template
const MUSK_CONF_TEMPLATE: &str = r#"# Musk Configuration
# ==================
# 
# Configure connection to your Elements/Liquid node.

# =============================================================================
# Network Configuration
# =============================================================================
[network]
# Network type - determines default ports and address formats
# Options:
#   - "regtest"   : Local development/testing (default port: 18884)
#   - "testnet"   : Liquid testnet (default port: 18892)
#   - "liquidv1"  : Liquid mainnet (default port: 7041)
network = "regtest"

# =============================================================================
# RPC Connection
# =============================================================================
[rpc]
# Elements/Liquid node RPC endpoint
url = "http://127.0.0.1:18884"

# RPC authentication credentials
# These should match your elements.conf settings
user = "user"
password = "password"

# =============================================================================
# Chain Configuration  
# =============================================================================
[chain]
# Genesis block hash - required for transaction signing (sighash computation)
# 
# If not provided, musk will fetch it automatically from the node.
# Specifying it here avoids an extra RPC call and ensures consistency.
#
# To find your genesis hash, run:
#   elements-cli getblockhash 0
#
# genesis_hash = "your-genesis-hash-here"
"#;

/// Execute the init command
///
/// Creates a musk directory and boilerplate musk.conf in the current directory.
///
/// # Errors
///
/// Returns an error if file/directory operations fail.
pub fn init_command(force: bool) -> Result<(), SprayError> {
    println!("{}", "Initializing Simplicity project...".cyan().bold());
    println!();

    let musk_dir = Path::new("musk");
    let musk_conf = Path::new("musk.conf");

    // Check if musk directory exists
    if musk_dir.exists() {
        if force {
            println!(
                "{} {}",
                "Directory exists:".yellow(),
                musk_dir.display()
            );
        } else {
            println!(
                "{} {} {}",
                "✓".green(),
                "Directory already exists:".dimmed(),
                musk_dir.display()
            );
        }
    } else {
        fs::create_dir(musk_dir)?;
        println!(
            "{} {} {}",
            "✓".green(),
            "Created directory:".dimmed(),
            musk_dir.display()
        );
    }

    // Check if musk.conf exists
    if musk_conf.exists() && !force {
        println!(
            "{} {} {}",
            "✓".green(),
            "Config already exists:".dimmed(),
            musk_conf.display()
        );
        println!();
        println!(
            "{}",
            "Use --force to overwrite existing musk.conf".yellow()
        );
    } else {
        if musk_conf.exists() {
            println!(
                "{} {}",
                "Overwriting:".yellow(),
                musk_conf.display()
            );
        }
        fs::write(musk_conf, MUSK_CONF_TEMPLATE)?;
        println!(
            "{} {} {}",
            "✓".green(),
            "Created config:".dimmed(),
            musk_conf.display()
        );
    }

    println!();
    println!("{}", "✓ Project initialized!".green().bold());
    println!();
    println!("{}", "Next steps:".bold());
    println!("  1. Edit {} with your node credentials", "musk.conf".cyan());
    println!("  2. Add your {} files to the {} directory", ".simf".cyan(), "musk/".cyan());
    println!("  3. Run {} to test your programs", "spray test".cyan());

    Ok(())
}

