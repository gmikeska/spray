# Spray

Testing workbench for Simplicity programs on Elements/Liquid networks.

## Overview

Spray provides a comprehensive CLI and testing framework for Simplicity programs. It uses [musk](../musk) for program operations and can work with both ephemeral regtest nodes (for isolated testing) and external nodes (for testnet/mainnet deployments).

## Installation

```bash
cd spray
cargo install --path .
```

## Commands

### `spray compile`

Compile a Simplicity program and output the compiled result.

```bash
# Basic compilation
spray compile program.simf

# With arguments
spray compile program.simf --args args.json

# With witness (to see final transaction size)
spray compile program.simf --witness witness.json

# Output in different formats
spray compile program.simf --output hex
spray compile program.simf --output base64
spray compile program.simf --output json  # default
```

The compile command outputs:
- Commitment Merkle Root (CMR)
- Program address (for receiving funds)
- Program size in bytes
- Compiled program (in requested format)

### `spray deploy`

Deploy a program to a network by funding its address.

```bash
# Deploy to local regtest (uses ephemeral node)
spray deploy program.simf --amount 100000000

# Deploy pre-compiled program
spray compile program.simf > compiled.json
spray deploy compiled.json --amount 50000000

# Deploy to testnet (requires config)
spray deploy program.simf \
    --network testnet \
    --config musk.toml \
    --amount 10000000

# With arguments
spray deploy program.simf --args args.json --amount 100000000
```

The deploy command:
1. Compiles the program (if `.simf`) or loads it (if `.json`)
2. Generates the program address
3. Sends funds to the address
4. Returns the funding UTXO (txid:vout)

### `spray redeem`

Spend from a program UTXO by providing a witness.

```bash
# Basic redemption (local regtest)
spray redeem <txid:vout> witness.json --compiled compiled.json

# With custom destination
spray redeem <txid:vout> witness.json \
    --compiled compiled.json \
    --dest ert1q...

# With custom fee
spray redeem <txid:vout> witness.json \
    --compiled compiled.json \
    --fee 5000

# On testnet
spray redeem <txid:vout> witness.json \
    --compiled compiled.json \
    --network testnet \
    --config musk.toml
```

The redeem command:
1. Fetches the UTXO from the network
2. Loads the compiled program
3. Builds a spending transaction
4. Computes the sighash
5. Finalizes with the provided witness
6. Broadcasts the transaction

### `spray test`

Test a program end-to-end (compile + deploy + redeem).

```bash
# Basic test
spray test --file program.simf --name "My test"

# With arguments and witness
spray test \
    --file program.simf \
    --args args.json \
    --witness witness.json \
    --name "P2PK test"

# With lock time
spray test \
    --file hodl_vault.simf \
    --witness hodl.json \
    --lock-time 1000 \
    --name "Time-locked vault"

# Verbose output
spray test --file program.simf --name "Test" --verbose
```

The test command:
1. Starts an ephemeral regtest node (or uses configured node)
2. Compiles the program
3. Deploys it (creates and funds a UTXO)
4. Attempts to spend from it with the witness
5. Reports success or failure

**Note**: The test command currently only supports `--network regtest` (uses ephemeral node).

## Network Backends

Spray supports two network backends:

### Ephemeral Regtest (Default)
- Automatically created for each test run
- Pre-funded with 2,100 BTC
- Isolated from external state
- Perfect for rapid testing

```bash
spray deploy program.simf  # Uses ephemeral node
spray test --file program.simf --name "Test"  # Uses ephemeral node
```

### External Node (via Config)
- Connect to testnet, liquid mainnet, or persistent regtest
- Requires a `musk.toml` config file
- Used for real deployments

```bash
spray deploy program.simf --network testnet --config musk.toml
spray redeem txid:vout witness.json --network testnet --config musk.toml
```

Example `musk.toml`:

```toml
network = "testnet"

[rpc]
url = "http://localhost:7041"
user = "user"
password = "password"

[chain]
genesis_hash = "a771da8e52ee6ad581ed1e9a99825e5b3b7992225534eaa2ae23244fe26ab1c1"
```

## File Formats

### Arguments Files

Arguments can be provided as JSON or TOML:

```json
{
  "pubkey": "02a1b2c3...",
  "amount": 100000000
}
```

```toml
pubkey = "02a1b2c3..."
amount = 100000000
```

### Witness Files

Witness values can be provided as JSON or TOML:

```json
{
  "signature": "304402..."
}
```

```toml
signature = "304402..."
```

## Programmatic Usage

```rust
use spray::{TestRunner, TestCase};
use musk::{Program, Arguments, WitnessValues};

// Create test environment
let runner = TestRunner::new()?;

// Load program
let program = Program::from_file("my_program.simf")?;
let compiled = program.instantiate(Arguments::default())?;

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
    TestCase::new(runner.env(), program1).name("Test 1"),
    TestCase::new(runner.env(), program2).name("Test 2"),
    TestCase::new(runner.env(), program3).name("Test 3"),
];

let results = runner.run_tests(tests);
```

## Architecture

Spray uses musk for all program operations, ensuring test coverage of production code paths:

```
CLI Commands
    ↓
┌─────────────────────────────────────────┐
│  compile → Compile & output program     │
│  deploy  → Fund program address         │
│  redeem  → Spend from program UTXO      │
│  test    → Full end-to-end test         │
└─────────────────────────────────────────┘
                ↓
        NetworkBackend
        ├─ Ephemeral (regtest)
        └─ External (testnet/liquid)
                ↓
         musk::Program
         musk::SpendBuilder
                ↓
    Elements Transaction → Broadcast
```

## Features

- **Multiple commands**: Compile, deploy, redeem, and test programs independently
- **Network flexibility**: Use ephemeral regtest for testing or external nodes for deployments
- **Automatic daemon management**: Regtest daemon started/stopped automatically
- **Sighash-based witnesses**: Witness functions receive the transaction sighash
- **Colored output**: Easy-to-read results with emoji indicators
- **Lock time/sequence support**: Test time-locked programs
- **Multiple file formats**: Support for JSON and TOML config files

## Examples

See the [SimplicityHL examples](../SimplicityHL/examples/) directory for program examples that work with spray.

Example workflow:

```bash
# 1. Compile and inspect
spray compile examples/cat.simf

# 2. Deploy to local regtest
spray deploy examples/cat.simf --amount 50000000
# Output: txid:vout

# 3. Create witness file (witness.json)
echo '{}' > witness.json

# 4. Compile to file for redemption
spray compile examples/cat.simf > compiled.json

# 5. Redeem (in a new terminal/session with the same node)
spray redeem <txid:vout> witness.json --compiled compiled.json

# Or, do it all in one command:
spray test --file examples/cat.simf --name "CAT test"
```

## License

MIT OR Apache-2.0
