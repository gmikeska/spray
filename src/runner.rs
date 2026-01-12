//! Test runner for executing multiple test cases

use crate::env::TestEnv;
use crate::error::SprayError;
use crate::test::{TestCase, TestResult};
use colored::Colorize;

/// Test runner for executing multiple test cases
pub struct TestRunner {
    env: TestEnv,
}

impl TestRunner {
    /// Create a new test runner
    pub fn new() -> Result<Self, SprayError> {
        let env = TestEnv::new()?;
        Ok(Self { env })
    }

    /// Get a reference to the test environment
    pub fn env(&self) -> &TestEnv {
        &self.env
    }

    /// Run a single test case
    pub fn run_test(&self, mut test: TestCase) -> TestResult {
        let test_name = test.name.clone();
        println!("{} {}", "⏳".yellow(), test_name.bold());

        // Create UTXO
        if let Err(e) = test.create_utxo() {
            let error = format!("Failed to create UTXO: {}", e);
            println!("{} {}: {}", "❌".red(), test_name.bold(), error.red());
            return TestResult::Failure { error };
        }

        // Generate blocks to confirm the funding transaction
        if let Err(e) = self.env.generate(1) {
            let error = format!("Failed to generate blocks: {}", e);
            println!("{} {}: {}", "❌".red(), test_name.bold(), error.red());
            return TestResult::Failure { error };
        }

        // Run the test
        match test.run() {
            Ok(TestResult::Success { txid }) => {
                println!("{} {} (txid: {})", "✅".green(), test_name.bold(), txid);
                TestResult::Success { txid }
            }
            Ok(TestResult::Failure { error }) => {
                println!("{} {}: {}", "❌".red(), test_name.bold(), error.red());
                TestResult::Failure { error }
            }
            Err(e) => {
                let error = e.to_string();
                println!("{} {}: {}", "❌".red(), test_name.bold(), error.red());
                TestResult::Failure { error }
            }
        }
    }

    /// Run multiple test cases
    pub fn run_tests(&self, tests: Vec<TestCase>) -> Vec<TestResult> {
        let mut results = Vec::new();

        println!("\n{}", "Running tests...".bold().cyan());
        println!("{}", "─".repeat(60).dimmed());

        for test in tests {
            results.push(self.run_test(test));
        }

        println!("{}", "─".repeat(60).dimmed());

        // Summary
        let success_count = results.iter().filter(|r| r.is_success()).count();
        let failure_count = results.iter().filter(|r| r.is_failure()).count();

        if failure_count == 0 {
            println!(
                "\n{} {} tests passed",
                "✓".green().bold(),
                success_count.to_string().green().bold()
            );
        } else {
            println!(
                "\n{} {} passed, {} failed",
                "⚠".yellow().bold(),
                success_count.to_string().green(),
                failure_count.to_string().red().bold()
            );
        }

        results
    }

    /// Generate blocks for lock time testing
    pub fn generate_blocks(&self, count: u32) -> Result<(), SprayError> {
        self.env.generate(count)
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new().expect("Failed to create test runner")
    }
}
