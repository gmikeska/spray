# Simplicity Contract Ecosystem

This workspace contains tools for developing, testing, and deploying Simplicity contracts on Elements/Liquid networks.

## Projects

### 1. [SimplicityHL](../SimplicityHL/)

The Simplicity high-level language compiler. Compiles `.simf` source files to Simplicity bytecode.

**Key Features:**
- Rust-like syntax for Simplicity contracts
- Type inference and safety
- Built-in jets and operations
- Witness and parameter support

### 2. [Musk](../musk/)

SDK for compiling, deploying, and spending Simplicity contracts in Rust applications.

**Key Features:**
- High-level contract API wrapping SimplicityHL
- Taproot address generation
- Transaction construction and signing
- Network-agnostic design (works with regtest, testnet, mainnet)

**Usage:**
```rust
use musk::{Contract, Arguments};

let contract = Contract::from_file("my_contract.simf")?;
let compiled = contract.instantiate(Arguments::default())?;
let address = compiled.address(&elements::AddressParams::ELEMENTS);
```

### 3. [Spray](.)

Testing workbench for Simplicity contracts using local regtest/testnet networks.

**Key Features:**
- Automated Elements regtest daemon management
- Test framework using musk APIs
- CLI for quick contract testing
- Support for time-locked contracts

**Usage:**
```bash
spray test --file contract.simf --name "My test"
```

## Architecture

```
┌─────────────────┐
│  SimplicityHL   │  ← Language & Compiler
│   (.simf → IR)  │
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│      Musk       │  ← SDK for Production
│  (Compile, Sign,│
│   Deploy, Spend)│
└────────┬────────┘
         │
         ↓
┌─────────────────┐
│     Spray       │  ← Testing Workbench
│   (Test Runner, │
│  Regtest Daemon)│
└─────────────────┘
```

## Design Philosophy

### Separation of Concerns

1. **SimplicityHL**: Focused purely on language and compilation
2. **Musk**: Production-ready SDK that wraps SimplicityHL with transaction handling
3. **Spray**: Testing tool that uses musk's "guts" to ensure test code matches production code

### Benefits

- **Musk** can be used in any Rust application (wallets, exchanges, dapps)
- **Spray** tests contracts using the exact same code path as production
- Changes to contract handling in **musk** automatically apply to **spray** tests
- Clean dependency graph: `SimplicityHL ← Musk ← Spray`

## Getting Started

### Prerequisites

- Rust 1.79.0 or later
- Elements daemon (for spray testing)

### Installation

```bash
# Build all projects
cd SimplicityHL && cargo build --release
cd ../musk && cargo build --release
cd ../spray && cargo build --release

# Install spray CLI
cd ../spray
cargo install --path .
```

### Quick Start: Testing a Contract

1. Write a Simplicity contract (`.simf` file):

```rust
// examples/hello.simf
fn main() {
    assert!(true);
}
```

2. Test it with spray:

```bash
spray test --file examples/hello.simf --name "Hello test"
```

3. Use it in your application with musk:

```rust
use musk::Contract;

let contract = Contract::from_file("examples/hello.simf")?;
let compiled = contract.instantiate(musk::Arguments::default())?;
let address = compiled.address(&musk::elements::AddressParams::ELEMENTS);
```

## Example Workflow

### Development & Testing Loop

```bash
# 1. Write contract
vim my_contract.simf

# 2. Test locally
spray test --file my_contract.simf --name "Test 1"

# 3. Test with witnesses
spray test --file my_contract.simf --witness test.wit --name "Test 2"

# 4. Test with lock time
spray test --file my_contract.simf --lock-time 1000 --name "Test 3"

# 5. Integrate into application using musk
```

### Production Deployment

```rust
use musk::{Contract, Arguments, SpendBuilder};

// Load contract
let contract = Contract::from_file("my_contract.simf")?;
let compiled = contract.instantiate(Arguments::default())?;

// Generate address for receiving funds
let address = compiled.address(&elements::AddressParams::ELEMENTS);

// Later, spend from the contract
let mut builder = SpendBuilder::new(compiled, utxo)
    .genesis_hash(genesis_hash);
builder.add_output_simple(destination, amount, asset);
builder.add_fee(fee, asset);

let sighash = builder.sighash_all()?;
let witness = generate_witness(sighash); // Your logic
let tx = builder.finalize(witness)?;

// Broadcast tx to network
```

## Examples

### Simple Contracts

See [SimplicityHL/examples/](../SimplicityHL/examples/) for contract examples:
- `cat.simf` - OP_CAT implementation
- `ctv.simf` - CheckTemplateVerify
- `p2pk.simf` - Pay to public key
- `p2pkh.simf` - Pay to public key hash
- `p2ms.simf` - Pay to multisig
- `hodl_vault.simf` - Time-locked vault
- `htlc.simf` - Hash time-locked contract
- And many more...

### Musk Integration

See [musk/examples/](../musk/examples/) for integration examples.

### Spray Tests

See [spray/examples/](./examples/) for testing examples.

## Data Flow

### Testing Flow (Spray)
```
spray test → TestRunner → TestEnv (Elements daemon)
                ↓
         TestCase → musk::Contract
                         ↓
                  musk::SpendBuilder
                         ↓
                    Transaction → Broadcast
```

### Production Flow (Musk)
```
Your App → musk::Contract → Compile
                ↓
         CompiledContract → Address Generation
                ↓
         SpendBuilder → Transaction Construction
                ↓
         NodeClient → Broadcast to Network
```

## Key Design Decisions

1. **Musk is network-agnostic**: The `NodeClient` trait allows musk to work with any Elements-compatible node, making it suitable for regtest, testnet, and mainnet.

2. **Spray uses musk exclusively**: All contract operations in spray go through musk, ensuring test coverage of production code paths.

3. **Witness functions are closures**: This pattern allows witnesses to be computed based on the sighash, enabling signature-based contracts.

4. **Re-export SimplicityHL types**: Musk re-exports necessary types from SimplicityHL (`Arguments`, `WitnessValues`, `Value`) so users don't need to depend on it directly.

## Contributing

Contributions are welcome! Please ensure:
- Code follows Rust idioms and style
- Tests pass for all affected components
- Documentation is updated

## License

- **SimplicityHL**: CC0-1.0 (Public Domain)
- **Musk**: MIT OR Apache-2.0
- **Spray**: MIT OR Apache-2.0

