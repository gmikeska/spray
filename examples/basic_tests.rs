//! Example: Testing Simplicity programs with spray
//!
//! Run with: cargo run --example basic_tests
//!
//! Note: This example requires a running elementsd daemon.

use musk::{Arguments, Program, WitnessValues};
use spray::{TestCase, TestRunner};

fn main() {
    println!("Running spray examples...\n");

    test_cat_program();
    test_p2pk_program();
    test_multiple_programs();

    println!("\nAll examples completed!");
}

fn test_cat_program() {
    let runner = TestRunner::new().expect("Failed to create test runner");

    // Load the OP_CAT example from SimplicityHL
    let program =
        Program::from_file("../SimplicityHL/examples/cat.simf").expect("Failed to load program");

    let compiled = program
        .instantiate(Arguments::default())
        .expect("Failed to compile program");

    let test = TestCase::new(runner.env(), compiled)
        .name("OP_CAT test")
        .witness(|_sighash| WitnessValues::default());

    let result = runner.run_test(test);
    assert!(result.is_success(), "OP_CAT test should succeed");
}

fn test_p2pk_program() {
    use musk::util;
    use musk::{Value, ValueConstructible, WitnessName};
    use std::collections::HashMap;

    let runner = TestRunner::new().expect("Failed to create test runner");

    // Load P2PK program
    let program =
        Program::from_file("../SimplicityHL/examples/p2pk.simf").expect("Failed to load program");

    // Create arguments with public key
    let secret_key = 1u32;
    let pubkey = util::xonly_public_key(secret_key);
    let mut args = HashMap::new();
    args.insert(
        WitnessName::from_str_unchecked("ALICE_PUBLIC_KEY"),
        Value::u256(musk::simplicityhl::num::U256::from_byte_array(pubkey)),
    );

    let compiled = program
        .instantiate(Arguments::from(args))
        .expect("Failed to compile program");

    let test = TestCase::new(runner.env(), compiled)
        .name("Pay to public key")
        .witness(move |sighash| {
            let signature = util::sign_schnorr(secret_key, sighash);
            let mut witness = HashMap::new();
            witness.insert(
                WitnessName::from_str_unchecked("ALICE_SIGNATURE"),
                Value::byte_array(signature),
            );
            WitnessValues::from(witness)
        });

    let result = runner.run_test(test);
    assert!(result.is_success(), "P2PK test should succeed");
}

fn test_multiple_programs() {
    let runner = TestRunner::new().expect("Failed to create test runner");

    // Test multiple programs
    let mut tests = vec![];

    // Test 1: CAT
    let cat_program = Program::from_file("../SimplicityHL/examples/cat.simf")
        .expect("Failed to load cat program");
    let cat_compiled = cat_program
        .instantiate(Arguments::default())
        .expect("Failed to compile cat");
    tests.push(
        TestCase::new(runner.env(), cat_compiled)
            .name("OP_CAT")
            .witness(|_| WitnessValues::default()),
    );

    // Test 2: CTV
    let ctv_program = Program::from_file("../SimplicityHL/examples/ctv.simf")
        .expect("Failed to load ctv program");
    let ctv_compiled = ctv_program
        .instantiate(Arguments::default())
        .expect("Failed to compile ctv");
    tests.push(
        TestCase::new(runner.env(), ctv_compiled)
            .name("CheckTemplateVerify")
            .witness(|_| WitnessValues::default()),
    );

    let results = runner.run_tests(tests);

    let all_success = results.iter().all(|r| r.is_success());
    assert!(all_success, "All tests should succeed");
}
