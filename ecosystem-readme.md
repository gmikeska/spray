# Simplicity Program Ecosystem

This workspace contains tools for developing, testing, and deploying Simplicity programs on Elements/Liquid networks.

## Projects

### 1. [SimplicityHL](../SimplicityHL/)

The Simplicity high-level language compiler. Compiles `.simf` source files to Simplicity bytecode.

**Key Features:**
- Rust-like syntax for Simplicity programs
- Type inference and safety
- Built-in jets and operations
- Witness and parameter support

### 2. [Musk](../musk/)

SDK for compiling, deploying, and spending Simplicity programs in Rust applications.

**Key Features:**
- High-level program API wrapping SimplicityHL
- Taproot address generation
- Transaction construction and signing
- Network-agnostic design (works with regtest, testnet, mainnet)

**Usage:**
```rust
use musk::{Program, Arguments};

let program = Program::from_file("my_program.simf")?;
let compiled = program.instantiate(Arguments::default())?;
let address = compiled.address(&elements::AddressParams::ELEMENTS);
```

### 3. [Spray](.)

Testing workbench and CLI for Simplicity programs on Elements/Liquid networks.

**Key Features:**
- Complete CLI with compile, deploy, redeem, and test commands
- Automated Elements regtest daemon management for testing
- Support for external nodes (testnet, liquid mainnet)
- Multiple file format support (JSON, TOML)
- Network-agnostic architecture using musk

**Usage:**
```bash
# Compile a program
spray compile program.simf

# Deploy to regtest
spray deploy program.simf --amount 100000000

# Redeem from a UTXO
spray redeem txid:vout witness.json --compiled compiled.json

# Full end-to-end test
spray test --file program.simf --name "My test"

# Deploy to testnet
spray deploy program.simf --network testnet --config musk.toml
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
- **Spray** tests programs using the exact same code path as production
- Changes to program handling in **musk** automatically apply to **spray** tests
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

### Quick Start: Testing a Program

1. Write a Simplicity program (`.simf` file):

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
use musk::Program;

let program = Program::from_file("examples/hello.simf")?;
let compiled = program.instantiate(musk::Arguments::default())?;
let address = compiled.address(&musk::elements::AddressParams::ELEMENTS);
```

## Example Workflow

### Development & Testing Loop

```bash
# 1. Write program
vim my_program.simf

# 2. Compile and inspect
spray compile my_program.simf

# 3. Test locally (end-to-end)
spray test --file my_program.simf --name "Test 1"

# 4. Test with witnesses
spray test --file my_program.simf --witness test.wit --name "Test 2"

# 5. Test with lock time
spray test --file my_program.simf --lock-time 1000 --name "Test 3"

# 6. Manual testing workflow
spray compile my_program.simf > compiled.json
spray deploy my_program.simf --amount 50000000  # Returns txid:vout
# ... create witness based on sighash ...
spray redeem <txid:vout> witness.json --compiled compiled.json

# 7. Deploy to testnet (with config)
spray deploy my_program.simf --network testnet --config musk.toml

# 8. Integrate into application using musk
```

### Production Deployment

```rust
use musk::{Program, Arguments, SpendBuilder};

// Load program
let program = Program::from_file("my_program.simf")?;
let compiled = program.instantiate(Arguments::default())?;

// Generate address for receiving funds
let address = compiled.address(&elements::AddressParams::ELEMENTS);

// Later, spend from the program
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

### Simple Programs

See [SimplicityHL/examples/](../SimplicityHL/examples/) for program examples:
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
┌─────────────────────────────────────────────────────────┐
│                    Spray CLI                            │
│  compile │ deploy │ redeem │ test                       │
└──────────┬──────────────────────────────────────────────┘
           │
           ↓
    ┌─────────────┐
    │ NetworkBackend │
    ├──────┬──────────┤
    │ Ephemeral     │  External
    │ (regtest)     │  (testnet/liquid)
    └──────┬──────────┘
           │
           ↓
    ┌─────────────┐
    │musk::Program│
    └──────┬──────┘
           │
           ↓
    ┌─────────────────┐
    │musk::SpendBuilder│
    └──────┬──────────┘
           │
           ↓
    ┌─────────────┐
    │ Transaction │ → Broadcast
    └─────────────┘
```

### Production Flow (Musk)

```
Your App → musk::Program → Compile
                ↓
         InstantiatedProgram → Address Generation
                ↓
         SpendBuilder → Transaction Construction
                ↓
         NodeClient → Broadcast to Network
```

## Key Design Decisions

1. **Musk is network-agnostic**: The `NodeClient` trait allows musk to work with any Elements-compatible node, making it suitable for regtest, testnet, and mainnet.

2. **Spray uses musk exclusively**: All program operations in spray go through musk, ensuring test coverage of production code paths.

3. **Witness functions are closures**: This pattern allows witnesses to be computed based on the sighash, enabling signature-based programs.

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

