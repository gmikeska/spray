//! Test case definition and execution

use crate::client::ElementsClient;
use crate::env::TestEnv;
use crate::error::SprayError;
use colored::Colorize;
use musk::client::{NodeClient, Utxo};
use musk::elements::{confidential, LockTime, Sequence};
use musk::{CompiledContract, SpendBuilder, WitnessValues};

/// Result of a test execution
#[derive(Debug, Clone)]
pub enum TestResult {
    Success { txid: musk::Txid },
    Failure { error: String },
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Success { .. })
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, TestResult::Failure { .. })
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
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Set the witness function
    pub fn witness<F>(mut self, f: F) -> Self
    where
        F: Fn([u8; 32]) -> WitnessValues + 'env,
    {
        self.witness_fn = Box::new(f);
        self
    }

    /// Set the lock time
    pub fn lock_time(mut self, lock_time: LockTime) -> Self {
        self.lock_time = lock_time;
        self
    }

    /// Set the sequence number
    pub fn sequence(mut self, sequence: Sequence) -> Self {
        self.sequence = sequence;
        self
    }

    /// Create a UTXO for this test by funding the contract address
    pub fn create_utxo(&mut self) -> Result<(), SprayError> {
        let client = ElementsClient::new(self.env.daemon());
        let address = self.contract.address(&musk::elements::AddressParams::ELEMENTS);

        println!("  {} {}", "Creating UTXO at:".dimmed(), address);

        // Send 1 BTC to the contract address
        let amount = 100_000_000; // 1 BTC in satoshis
        let txid = client
            .send_to_address(&address, amount)
            .map_err(|e| SprayError::TestError(e.to_string()))?;

        self.funding_txid = Some(txid);
        println!("  {} {}", "Funding txid:".dimmed(), txid);

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

        let address = self.contract.address(&musk::elements::AddressParams::ELEMENTS);
        let script = address.script_pubkey();

        // Find the output that matches our script
        for (vout, txout) in tx.output.iter().enumerate() {
            if txout.script_pubkey == script {
                let amount = match txout.value {
                    confidential::Value::Explicit(amt) => amt,
                    _ => continue,
                };

                return Ok(Utxo {
                    txid,
                    vout: vout as u32,
                    amount,
                    script_pubkey: txout.script_pubkey.clone(),
                    asset: txout.asset,
                });
            }
        }

        Err(SprayError::TestError("UTXO not found in transaction".into()))
    }

    /// Run the test
    pub fn run(self) -> Result<TestResult, SprayError> {
        let client = ElementsClient::new(self.env.daemon());

        // Get the UTXO
        let utxo = self.get_utxo()?;

        // Get the asset
        let asset = match utxo.asset {
            confidential::Asset::Explicit(id) => id,
            _ => return Err(SprayError::TestError("Non-explicit asset".into())),
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
            .map_err(|e| SprayError::TestError(format!("Failed to broadcast: {}", e)))?;

        Ok(TestResult::Success { txid })
    }
}

