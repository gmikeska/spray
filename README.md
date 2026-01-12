# Spray

Testing workbench for Simplicity contracts on Elements/Liquid regtest networks.

## Overview

Spray provides a testing framework and CLI for testing Simplicity contracts locally. It uses [musk](../musk) for contract operations and manages an Elements regtest daemon for isolated testing.

## Installation

```bash
cd spray
cargo install --path .
```

## Usage

### CLI

Test a contract:

```bash
spray test --file ../SimplicityHL/examples/cat.simf --name "OP_CAT test"
```

With arguments:

```bash
spray test \
    --file ../SimplicityHL/examples/p2pk.simf \
    --args p2pk.args \
    --witness p2pk.wit \
    --name "Pay to public key"
```

With lock time:

```bash
spray test \
    --file ../SimplicityHL/examples/hodl_vault.simf \
    --witness hodl_vault.wit \
    --lock-time 1000 \
    --name "HODL vault"
```

### Programmatic Usage

```rust
use spray::{TestRunner, TestCase};
use musk::{Contract, Arguments, WitnessValues};

// Create test environment
let runner = TestRunner::new()?;

// Load contract
let contract = Contract::from_file("my_contract.simf")?;
let compiled = contract.instantiate(Arguments::default())?;

// Create test case
let test = TestCase::new(runner.env(), compiled)
    .name("My test")
    .witness(|sighash| {
        // Generate witness values based on sighash
        let signature = sign_schnorr(secret_key, sighash);
        // ... build WitnessValues
        WitnessValues::default()
    });

// Run test
let result = runner.run_test(test);
assert!(result.is_success());
```

### Multiple Tests

```rust
let runner = TestRunner::new()?;

let tests = vec![
    TestCase::new(runner.env(), contract1).name("Test 1"),
    TestCase::new(runner.env(), contract2).name("Test 2"),
    TestCase::new(runner.env(), contract3).name("Test 3"),
];

let results = runner.run_tests(tests);
```

## Architecture

Spray uses musk for all contract operations, ensuring test coverage of production code paths:

```
CLI → TestRunner → TestEnv (ElementsD)
                ↓
         TestCase → musk::Contract
                           ↓
                    musk::SpendBuilder
                           ↓
                      Transaction → Broadcast
```

## Features

- **Automatic daemon management**: Elements regtest daemon is started and stopped automatically
- **Test isolation**: Each test run uses a fresh regtest environment
- **Sighash-based witnesses**: Witness functions receive the transaction sighash for signature generation
- **Colored output**: Easy-to-read test results with emoji indicators
- **Lock time/sequence support**: Test time-locked contracts

## Examples

See the [SimplicityHL examples](../SimplicityHL/examples/) directory for contract examples that work with spray.

## License

MIT OR Apache-2.0

