//! Test case definition and execution
//!
//! This module provides the [`TestCase`] builder for defining contract tests
//! and the [`TestResult`] enum for test outcomes.

use crate::client::ElementsClient;
use crate::env::TestEnv;
use crate::error::SprayError;
use colored::Colorize;
use musk::client::{NodeClient, Utxo};
use musk::elements::{confidential, LockTime, Sequence};
use musk::{CompiledContract, SpendBuilder, WitnessValues};

/// Result of a test execution
///
/// # Example
///
/// ```
/// use spray::TestResult;
/// use musk::elements::Txid;
/// use std::str::FromStr;
///
/// let txid = Txid::from_str(
///     "0000000000000000000000000000000000000000000000000000000000000000"
/// ).unwrap();
///
/// let success = TestResult::Success { txid };
/// assert!(success.is_success());
/// assert!(!success.is_failure());
///
/// let failure = TestResult::Failure { error: "test failed".into() };
/// assert!(failure.is_failure());
/// assert!(!failure.is_success());
/// ```
#[derive(Debug, Clone)]
pub enum TestResult {
    /// Test passed, contains the spending transaction ID
    Success { txid: musk::Txid },
    /// Test failed, contains the error message
    Failure { error: String },
}

impl TestResult {
    /// Returns `true` if this is a successful test result
    ///
    /// # Example
    ///
    /// ```
    /// use spray::TestResult;
    /// use musk::elements::Txid;
    /// use std::str::FromStr;
    ///
    /// let txid = Txid::from_str(
    ///     "0000000000000000000000000000000000000000000000000000000000000000"
    /// ).unwrap();
    /// let result = TestResult::Success { txid };
    /// assert!(result.is_success());
    /// ```
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Returns `true` if this is a failed test result
    ///
    /// # Example
    ///
    /// ```
    /// use spray::TestResult;
    ///
    /// let result = TestResult::Failure { error: "assertion failed".into() };
    /// assert!(result.is_failure());
    /// ```
    #[must_use]
    pub const fn is_failure(&self) -> bool {
        matches!(self, Self::Failure { .. })
    }
}

/// A test case for a Simplicity contract
pub struct TestCase<'env> {
    pub name: String,
    env: &'env TestEnv,
    contract: CompiledContract,
    witness_fn: Box<dyn Fn([u8; 32]) -> WitnessValues + 'env>,
    lock_time: LockTime,
    sequence: Sequence,
    funding_txid: Option<musk::Txid>,
}

impl<'env> TestCase<'env> {
    /// Create a new test case
    pub fn new(env: &'env TestEnv, contract: CompiledContract) -> Self {
        Self {
            name: "Unnamed test".to_string(),
            env,
            contract,
            witness_fn: Box::new(|_| WitnessValues::default()),
            lock_time: LockTime::ZERO,
            sequence: Sequence::MAX,
            funding_txid: None,
        }
    }

    /// Set the test name
    #[must_use]
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set the witness function
    #[must_use]
    pub fn witness<F>(mut self, f: F) -> Self
    where
        F: Fn([u8; 32]) -> WitnessValues + 'env,
    {
        self.witness_fn = Box::new(f);
        self
    }

    /// Set the lock time
    #[must_use]
    pub const fn lock_time(mut self, lock_time: LockTime) -> Self {
        self.lock_time = lock_time;
        self
    }

    /// Set the sequence number
    #[must_use]
    pub const fn sequence(mut self, sequence: Sequence) -> Self {
        self.sequence = sequence;
        self
    }

    /// Create a UTXO for this test by funding the contract address
    ///
    /// # Errors
    ///
    /// Returns an error if sending to the contract address fails.
    pub fn create_utxo(&mut self) -> Result<(), SprayError> {
        let client = ElementsClient::new(self.env.daemon());
        let address = self
            .contract
            .address(&musk::elements::AddressParams::ELEMENTS);

        println!("  {} {address}", "Creating UTXO at:".dimmed());

        // Send 1 BTC to the contract address
        let amount = 100_000_000; // 1 BTC in satoshis
        let txid = client
            .send_to_address(&address, amount)
            .map_err(|e| SprayError::TestError(e.to_string()))?;

        self.funding_txid = Some(txid);
        println!("  {} {txid}", "Funding txid:".dimmed());

        Ok(())
    }

    /// Get the UTXO for spending
    fn get_utxo(&self) -> Result<Utxo, SprayError> {
        let txid = self
            .funding_txid
            .ok_or_else(|| SprayError::TestError("Test UTXO not created".into()))?;

        let client = ElementsClient::new(self.env.daemon());
        let tx = client
            .get_transaction(&txid)
            .map_err(|e| SprayError::TestError(e.to_string()))?;

        let address = self
            .contract
            .address(&musk::elements::AddressParams::ELEMENTS);
        let script = address.script_pubkey();

        // Find the output that matches our script
        for (vout, txout) in tx.output.iter().enumerate() {
            if txout.script_pubkey == script {
                let confidential::Value::Explicit(amount) = txout.value else {
                    continue;
                };

                return Ok(Utxo {
                    txid,
                    #[allow(clippy::cast_possible_truncation)]
                    vout: vout as u32,
                    amount,
                    script_pubkey: txout.script_pubkey.clone(),
                    asset: txout.asset,
                });
            }
        }

        Err(SprayError::TestError(
            "UTXO not found in transaction".into(),
        ))
    }

    /// Run the test
    ///
    /// # Errors
    ///
    /// Returns an error if the UTXO cannot be retrieved, the transaction
    /// cannot be built, or broadcasting fails.
    pub fn run(self) -> Result<TestResult, SprayError> {
        let client = ElementsClient::new(self.env.daemon());

        // Get the UTXO
        let utxo = self.get_utxo()?;

        // Get the asset
        let confidential::Asset::Explicit(asset) = utxo.asset else {
            return Err(SprayError::TestError("Non-explicit asset".into()));
        };

        // Build the spending transaction
        let mut builder = SpendBuilder::new(self.contract.clone(), utxo)
            .genesis_hash(self.env.genesis_hash())
            .lock_time(self.lock_time)
            .sequence(self.sequence);

        // Add outputs
        let destination = client
            .get_new_address()
            .map_err(|e| SprayError::TestError(e.to_string()))?;
        let output_amount = 99_997_000; // Leave room for fee
        let fee_amount = 3_000;

        builder.add_output_simple(destination.script_pubkey(), output_amount, asset);
        builder.add_fee(fee_amount, asset);

        // Compute sighash
        let sighash = builder
            .sighash_all()
            .map_err(|e| SprayError::TestError(e.to_string()))?;

        // Generate witness values
        let witness_values = (self.witness_fn)(sighash);

        // Finalize the transaction
        let tx = builder
            .finalize(witness_values)
            .map_err(|e| SprayError::TestError(e.to_string()))?;

        // Broadcast
        let txid = client
            .broadcast(&tx)
            .map_err(|e| SprayError::TestError(format!("Failed to broadcast: {e}")))?;

        Ok(TestResult::Success { txid })
    }
}
