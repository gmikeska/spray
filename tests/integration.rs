//! Integration tests for daemon-dependent functionality
//!
//! These tests require a running elementsd daemon and are marked as `#[ignore]`
//! by default. Run with `cargo test -- --ignored` to execute them.

use musk::{Arguments, Program, WitnessValues};
use spray::{TestCase, TestEnv, TestRunner};

/// Test that TestEnv successfully creates and configures a wallet
#[test]
#[ignore = "Requires elementsd daemon"]
fn test_env_creates_wallet() {
    let env = TestEnv::new().expect("Failed to create test environment");

    // If we got here, wallet was created successfully
    // The genesis hash should be valid
    let genesis = env.genesis_hash();
    assert_ne!(
        genesis.to_string(),
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
}

/// Test that TestEnv can generate blocks
#[test]
#[ignore = "Requires elementsd daemon"]
fn test_env_generates_blocks() {
    let env = TestEnv::new().expect("Failed to create test environment");

    // Generate some blocks
    env.generate(10).expect("Failed to generate blocks");

    // If we got here, block generation works
}

/// Test the TestCase builder pattern
#[test]
#[ignore = "Requires elementsd daemon"]
fn test_testcase_builder_pattern() {
    let runner = TestRunner::new().expect("Failed to create test runner");

    let program =
        Program::from_source("fn main() { assert!(true); }").expect("Failed to parse program");
    let compiled = program
        .instantiate(Arguments::default())
        .expect("Failed to compile");

    // Test builder methods
    let test = TestCase::new(runner.env(), compiled)
        .name("Builder test")
        .witness(|_sighash| WitnessValues::default())
        .lock_time(musk::elements::LockTime::ZERO)
        .sequence(musk::elements::Sequence::MAX);

    // Name should be set
    assert_eq!(test.name, "Builder test");
}

/// Test full compile workflow without daemon
#[test]
fn test_compile_workflow_no_daemon() {
    use spray::compiled::CompiledOutput;

    // Compile a program
    let program =
        Program::from_source("fn main() { assert!(true); }").expect("Failed to parse program");
    let compiled = program
        .instantiate(Arguments::default())
        .expect("Failed to compile");

    // Create output
    let output =
        CompiledOutput::from_compiled(&compiled, Some("fn main() { assert!(true); }".to_string()));

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&output).expect("Failed to serialize");

    // Deserialize back
    let parsed: CompiledOutput = serde_json::from_str(&json).expect("Failed to deserialize");

    // Verify
    assert_eq!(parsed.cmr, output.cmr);
    assert!(parsed.source.is_some());
}

/// Test TestRunner can run a simple program
#[test]
#[ignore = "Requires elementsd daemon"]
fn test_runner_executes_simple_program() {
    let runner = TestRunner::new().expect("Failed to create test runner");

    let program =
        Program::from_source("fn main() { assert!(true); }").expect("Failed to parse program");
    let compiled = program
        .instantiate(Arguments::default())
        .expect("Failed to compile");

    let test = TestCase::new(runner.env(), compiled)
        .name("Simple program test")
        .witness(|_| WitnessValues::default());

    let result = runner.run_test(test);
    assert!(result.is_success(), "Simple program should succeed");
}

/// Test network backend creation for regtest
#[test]
#[ignore = "Requires elementsd daemon"]
fn test_network_backend_regtest() {
    use musk::Network;
    use spray::network::create_backend;

    let backend = create_backend(Network::Regtest, None);
    assert!(backend.is_ok(), "Should create ephemeral regtest backend");
}
