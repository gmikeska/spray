//! Compile command implementation

use crate::compiled::CompiledOutput;
use crate::error::SprayError;
use crate::file_loader;
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Output format for compiled programs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Base64,
    Hex,
}

impl OutputFormat {
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "json" => Some(Self::Json),
            "base64" => Some(Self::Base64),
            "hex" => Some(Self::Hex),
            _ => None,
        }
    }
}

/// Execute the compile command
///
/// # Errors
///
/// Returns an error if compilation fails or file operations fail.
#[allow(clippy::too_many_lines)]
pub fn compile_command(
    file: &Path,
    args: Option<PathBuf>,
    witness: Option<PathBuf>,
    output_format: OutputFormat,
    network: musk::Network,
) -> Result<(), SprayError> {
    println!("{}", "Compiling Simplicity program...".cyan().bold());
    println!();

    // Load program
    println!("{} {}", "Loading program from:".dimmed(), file.display());
    let source = std::fs::read_to_string(file)?;
    let program = musk::Program::from_source(&source)?;

    // Load arguments if provided
    let arguments = if let Some(args_path) = args {
        println!(
            "{} {}",
            "Loading arguments from:".dimmed(),
            args_path.display()
        );
        file_loader::load_arguments(&args_path)?
    } else {
        musk::Arguments::default()
    };

    // Compile program
    println!("{}", "Compiling...".dimmed());
    let compiled = program.instantiate(arguments)?;

    // Get CMR
    let cmr = compiled.cmr();
    let cmr_hex = hex::encode(cmr.as_ref());

    // Get address for the network
    let address = compiled.address(network.address_params());

    // Create output based on whether witness was provided
    let output = if let Some(witness_path) = witness {
        println!(
            "{} {}",
            "Loading witness from:".dimmed(),
            witness_path.display()
        );
        let witness_values = file_loader::load_witness(&witness_path)?;
        let satisfied = compiled.satisfy(witness_values)?;
        CompiledOutput::from_satisfied(&satisfied, &compiled, Some(source))
    } else {
        CompiledOutput::from_compiled(&compiled, Some(source))
    };

    println!();
    println!("{}", "✓ Compilation successful!".green().bold());
    println!();

    // Display basic info
    println!("{}", "Program Information:".bold());
    println!("  {} {}", "CMR:".bold(), cmr_hex);
    println!("  {} {}", "Address:".bold(), address);
    println!("  {} {} bytes", "Size:".bold(), output.program_size);

    if let Some(ref witness) = output.witness {
        use base64::{engine::general_purpose::STANDARD, Engine};
        if let Ok(witness_bytes) = STANDARD.decode(witness) {
            println!("  {} {} bytes", "Witness size:".bold(), witness_bytes.len());
        }
    }

    println!();

    // Output in requested format
    match output_format {
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&output)?;
            println!("{json}");
        }
        OutputFormat::Base64 => {
            println!("{}", "Program (base64):".bold());
            println!("{}", output.program);
            if let Some(witness) = output.witness {
                println!();
                println!("{}", "Witness (base64):".bold());
                println!("{witness}");
            }
        }
        OutputFormat::Hex => {
            use base64::{engine::general_purpose::STANDARD, Engine};
            let program_bytes = STANDARD
                .decode(&output.program)
                .map_err(|e| SprayError::ParseError(format!("Failed to decode program: {e}")))?;
            println!("{}", "Program (hex):".bold());
            println!("{}", hex::encode(&program_bytes));
            if let Some(witness) = output.witness {
                let witness_bytes = STANDARD.decode(&witness).map_err(|e| {
                    SprayError::ParseError(format!("Failed to decode witness: {e}"))
                })?;
                println!();
                println!("{}", "Witness (hex):".bold());
                println!("{}", hex::encode(&witness_bytes));
            }
        }
    }

    Ok(())
}

// Add hex module
#[doc(hidden)]
mod hex {
    use std::fmt::Write;

    pub fn encode(bytes: &[u8]) -> String {
        bytes
            .iter()
            .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            })
    }
}
