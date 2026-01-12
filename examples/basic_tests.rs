//! Example: Testing Simplicity contracts with spray
//!
//! Run with: cargo test --example basic_tests

use musk::{Contract, Arguments, WitnessValues};
use spray::{TestRunner, TestCase};

#[test]
fn test_cat_contract() {
    let runner = TestRunner::new().expect("Failed to create test runner");

    // Load the OP_CAT example from SimplicityHL
    let contract = Contract::from_file("../SimplicityHL/examples/cat.simf")
        .expect("Failed to load contract");

    let compiled = contract
        .instantiate(Arguments::default())
        .expect("Failed to compile contract");

    let test = TestCase::new(runner.env(), compiled)
        .name("OP_CAT test")
        .witness(|_sighash| WitnessValues::default());

    let result = runner.run_test(test);
    assert!(result.is_success(), "OP_CAT test should succeed");
}

#[test]
fn test_p2pk_contract() {
    use musk::util;
    use musk::{Value, WitnessName};
    use std::collections::HashMap;

    let runner = TestRunner::new().expect("Failed to create test runner");

    // Load P2PK contract
    let contract = Contract::from_file("../SimplicityHL/examples/p2pk.simf")
        .expect("Failed to load contract");

    // Create arguments with public key
    let secret_key = 1u32;
    let pubkey = util::xonly_public_key(secret_key);
    let mut args = HashMap::new();
    args.insert(
        WitnessName::from_str_unchecked("ALICE_PUBLIC_KEY"),
        Value::u256(musk::simplicityhl::num::U256::from_byte_array(pubkey)),
    );

    let compiled = contract
        .instantiate(Arguments::from(args))
        .expect("Failed to compile contract");

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

#[test]
fn test_multiple_contracts() {
    let runner = TestRunner::new().expect("Failed to create test runner");

    // Test multiple contracts
    let mut tests = vec![];

    // Test 1: CAT
    let cat_contract = Contract::from_file("../SimplicityHL/examples/cat.simf")
        .expect("Failed to load cat contract");
    let cat_compiled = cat_contract
        .instantiate(Arguments::default())
        .expect("Failed to compile cat");
    tests.push(
        TestCase::new(runner.env(), cat_compiled)
            .name("OP_CAT")
            .witness(|_| WitnessValues::default()),
    );

    // Test 2: CTV
    let ctv_contract = Contract::from_file("../SimplicityHL/examples/ctv.simf")
        .expect("Failed to load ctv contract");
    let ctv_compiled = ctv_contract
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

