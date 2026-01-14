//! Spray - Testing workbench for Simplicity programs
//!
//! This crate provides tools for testing Simplicity programs locally
//! on a regtest or testnet Liquid network node.
//!
//! # Example
//!
//! ```ignore
//! use spray::{TestEnv, TestCase};
//!
//! let env = TestEnv::new()?;
//! let program = musk::Program::from_file("my_program.simf")?;
//! let compiled = program.instantiate(musk::Arguments::default())?;
//!
//! let test = TestCase::new(&env, compiled)
//!     .name("My test")
//!     .witness(|sighash| {
//!         // Generate witness values based on sighash
//!         musk::WitnessValues::default()
//!     });
//!
//! let result = test.run()?;
//! ```

pub mod client;
pub mod compiled;
pub mod env;
pub mod error;
pub mod file_loader;
pub mod network;
pub mod runner;
pub mod test;

pub mod commands;

// Re-export main types
pub use compiled::CompiledOutput;
pub use env::TestEnv;
pub use error::SprayError;
pub use network::{create_backend, NetworkBackend};
pub use runner::TestRunner;
pub use test::{TestCase, TestResult};

// Re-export musk for convenience
pub use musk;
