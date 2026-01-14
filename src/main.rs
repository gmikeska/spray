//! Spray CLI - Testing workbench for Simplicity programs

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use spray::{commands, musk, SprayError, TestCase, TestRunner};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "spray")]
#[command(about = "Testing workbench for Simplicity programs", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum NetworkArg {
    Regtest,
    Testnet,
    Liquid,
}

impl From<NetworkArg> for musk::Network {
    fn from(arg: NetworkArg) -> Self {
        match arg {
            NetworkArg::Regtest => Self::Regtest,
            NetworkArg::Testnet => Self::Testnet,
            NetworkArg::Liquid => Self::Liquid,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Json,
    Base64,
    Hex,
}

#[derive(Subcommand)]
enum Commands {
    /// Compile a Simplicity program
    Compile {
        /// Path to the .simf program file
        file: PathBuf,

        /// Path to arguments file (JSON or TOML)
        #[arg(short, long)]
        args: Option<PathBuf>,

        /// Path to witness file (JSON or TOML)
        #[arg(short, long)]
        witness: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value = "json")]
        output: OutputFormat,

        /// Network (for address generation)
        #[arg(short, long, value_enum, default_value = "regtest")]
        network: NetworkArg,
    },

    /// Deploy a program to the network
    Deploy {
        /// Path to .simf source file or compiled .json file
        file: PathBuf,

        /// Path to arguments file (JSON or TOML, for .simf files only)
        #[arg(short, long)]
        args: Option<PathBuf>,

        /// Amount to fund (in satoshis)
        #[arg(long, default_value = "100000000")]
        amount: u64,

        /// Asset ID (hex)
        #[arg(long)]
        asset: Option<String>,

        /// Network
        #[arg(short, long, value_enum, default_value = "regtest")]
        network: NetworkArg,

        /// Config file (required for testnet/liquid)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Redeem from a program UTXO
    Redeem {
        /// UTXO reference in format "txid:vout"
        utxo: String,

        /// Path to witness file (JSON or TOML)
        witness: PathBuf,

        /// Path to compiled program file (.json with source)
        #[arg(short, long)]
        compiled: Option<PathBuf>,

        /// Destination address (defaults to new address from wallet)
        #[arg(short, long)]
        dest: Option<String>,

        /// Fee in satoshis
        #[arg(short, long, default_value = "3000")]
        fee: u64,

        /// Network
        #[arg(short, long, value_enum, default_value = "regtest")]
        network: NetworkArg,

        /// Config file (required for testnet/liquid)
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Test a Simplicity program (compile + deploy + redeem)
    Test {
        /// Path to the .simf program file
        #[arg(short, long)]
        file: PathBuf,

        /// Path to arguments file (JSON or TOML)
        #[arg(short, long)]
        args: Option<PathBuf>,

        /// Path to witness file (JSON or TOML)
        #[arg(short, long)]
        witness: Option<PathBuf>,

        /// Test name
        #[arg(short, long, default_value = "Program test")]
        name: String,

        /// Lock time for the spending transaction
        #[arg(long)]
        lock_time: Option<u32>,

        /// Sequence number for the spending transaction
        #[arg(long)]
        sequence: Option<u32>,

        /// Network (currently only regtest is supported for test command)
        #[arg(long, value_enum, default_value = "regtest")]
        network: NetworkArg,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start an interactive REPL
    Repl,

    /// Manage Elements regtest daemon
    Daemon,
}

#[allow(clippy::too_many_lines)]
fn main() -> Result<(), SprayError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compile {
            file,
            args,
            witness,
            output,
            network,
        } => {
            let output_fmt = match output {
                OutputFormat::Json => commands::compile::OutputFormat::Json,
                OutputFormat::Base64 => commands::compile::OutputFormat::Base64,
                OutputFormat::Hex => commands::compile::OutputFormat::Hex,
            };
            commands::compile_command(&file, args, witness, output_fmt, network.into())?;
        }

        Commands::Deploy {
            file,
            args,
            amount,
            asset,
            network,
            config,
        } => {
            commands::deploy_command(&file, args, Some(amount), asset, network.into(), config)?;
        }

        Commands::Redeem {
            utxo,
            witness,
            compiled,
            dest,
            fee,
            network,
            config,
        } => {
            commands::redeem_command(
                &utxo,
                &witness,
                compiled,
                dest,
                Some(fee),
                network.into(),
                config,
            )?;
        }

        Commands::Test {
            file,
            args,
            witness,
            name,
            lock_time,
            sequence,
            network,
            verbose,
        } => {
            // Only regtest is supported for test command
            if !matches!(network, NetworkArg::Regtest) {
                return Err(SprayError::ConfigError(
                    "Test command currently only supports --network regtest".into(),
                ));
            }

            if verbose {
                println!("{}", "Initializing test environment...".dimmed());
            }

            let runner = TestRunner::new()?;

            if verbose {
                println!("{}", "Loading program...".dimmed());
            }

            // Load program
            let program = musk::Program::from_file(&file)?;

            // Load arguments if provided
            let arguments = if let Some(args_path) = args {
                if verbose {
                    println!(
                        "{} {}",
                        "Loading arguments from:".dimmed(),
                        args_path.display()
                    );
                }
                spray::file_loader::load_arguments(&args_path)?
            } else {
                musk::Arguments::default()
            };

            // Compile program
            let compiled = program.instantiate(arguments)?;

            // Create witness function
            let witness_fn: Box<dyn Fn([u8; 32]) -> musk::WitnessValues> =
                if let Some(witness_path) = witness {
                    // Load witness from file
                    let witness_values = spray::file_loader::load_witness(&witness_path)?;
                    Box::new(move |_sighash| witness_values.clone())
                } else {
                    // Empty witness
                    Box::new(|_sighash| musk::WitnessValues::default())
                };

            // Create test case
            let mut test = TestCase::new(runner.env(), compiled).name(&name);

            test = test.witness(witness_fn);

            if let Some(lt) = lock_time {
                test = test.lock_time(musk::elements::LockTime::from_consensus(lt));
            }

            if let Some(seq) = sequence {
                test = test.sequence(musk::elements::Sequence::from_consensus(seq));
            }

            // Run test
            let result = runner.run_test(test);

            if result.is_failure() {
                std::process::exit(1);
            }
        }

        Commands::Repl => {
            println!("{}", "Interactive REPL not yet implemented".yellow());
            println!("Use 'spray test --help' to see testing options");
        }

        Commands::Daemon => {
            println!("{}", "Daemon management not yet implemented".yellow());
            println!("The daemon is automatically started when running tests");
        }
    }

    Ok(())
}
