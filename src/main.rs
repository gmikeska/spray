//! Spray CLI - Testing workbench for Simplicity contracts

use clap::{Parser, Subcommand};
use colored::Colorize;
use spray::{musk, SprayError, TestCase, TestRunner};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "spray")]
#[command(about = "Testing workbench for Simplicity contracts", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a Simplicity contract
    Test {
        /// Path to the .simf contract file
        #[arg(short, long)]
        file: PathBuf,

        /// Path to arguments JSON file
        #[arg(short, long)]
        args: Option<PathBuf>,

        /// Path to witness JSON file
        #[arg(short, long)]
        witness: Option<PathBuf>,

        /// Test name
        #[arg(short, long, default_value = "Contract test")]
        name: String,

        /// Lock time for the spending transaction
        #[arg(long)]
        lock_time: Option<u32>,

        /// Sequence number for the spending transaction
        #[arg(long)]
        sequence: Option<u32>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Start an interactive REPL
    Repl,

    /// Manage Elements regtest daemon
    Daemon,
}

fn main() -> Result<(), SprayError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            file,
            args,
            witness,
            name,
            lock_time,
            sequence,
            verbose,
        } => {
            if verbose {
                println!("{}", "Initializing test environment...".dimmed());
            }

            let runner = TestRunner::new()?;

            if verbose {
                println!("{}", "Loading contract...".dimmed());
            }

            // Load contract
            let contract = musk::Contract::from_file(&file)?;

            // Load arguments if provided
            let arguments = if let Some(args_path) = args {
                if verbose {
                    println!("{} {:?}", "Loading arguments from:".dimmed(), args_path);
                }
                let args_json = std::fs::read_to_string(args_path)?;
                serde_json::from_str(&args_json)?
            } else {
                musk::Arguments::default()
            };

            // Compile contract
            let compiled = contract.instantiate(arguments)?;

            // Create witness function
            let witness_fn: Box<dyn Fn([u8; 32]) -> musk::WitnessValues> =
                if let Some(witness_path) = witness {
                    // Load witness from file
                    let witness_json = std::fs::read_to_string(witness_path)?;
                    let witness_values: musk::WitnessValues = serde_json::from_str(&witness_json)?;
                    Box::new(move |_sighash| witness_values.clone())
                } else {
                    // Empty witness
                    Box::new(|_sighash| musk::WitnessValues::default())
                };

            // Create test case
            let mut test = TestCase::new(runner.env(), compiled).name(&name);

            test = test.witness(move |sighash| witness_fn(sighash));

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
