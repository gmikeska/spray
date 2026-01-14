//! Unit tests for CompiledOutput serialization

use spray::compiled::CompiledOutput;
use std::collections::HashMap;

#[test]
fn test_compiled_output_serialization() {
    let output = CompiledOutput {
        cmr: "deadbeef".to_string(),
        program: "SGVsbG8gV29ybGQ=".to_string(), // "Hello World" in base64
        witness: Some("dGVzdA==".to_string()),    // "test" in base64
        witness_types: HashMap::new(),
        program_size: 11,
        source: Some("fn main() { assert!(true); }".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&output).expect("Failed to serialize");

    // Deserialize back
    let deserialized: CompiledOutput =
        serde_json::from_str(&json).expect("Failed to deserialize");

    assert_eq!(deserialized.cmr, output.cmr);
    assert_eq!(deserialized.program, output.program);
    assert_eq!(deserialized.witness, output.witness);
    assert_eq!(deserialized.program_size, output.program_size);
    assert_eq!(deserialized.source, output.source);
}

#[test]
fn test_from_compiled_creates_valid_output() {
    let contract = musk::Contract::from_source("fn main() { assert!(true); }")
        .expect("Failed to parse contract");
    let compiled = contract
        .instantiate(musk::Arguments::default())
        .expect("Failed to compile");

    let output = CompiledOutput::from_compiled(&compiled, Some("fn main() { assert!(true); }".to_string()));

    // CMR should be a valid hex string (64 chars for 32 bytes)
    assert_eq!(output.cmr.len(), 64);
    assert!(output.cmr.chars().all(|c| c.is_ascii_hexdigit()));

    // Program should be valid base64
    assert!(!output.program.is_empty());

    // No witness for unsatisfied contract
    assert!(output.witness.is_none());

    // Source should be preserved
    assert_eq!(output.source, Some("fn main() { assert!(true); }".to_string()));
}

#[test]
fn test_decode_program_base64() {
    let output = CompiledOutput {
        cmr: "deadbeef".to_string(),
        program: "SGVsbG8gV29ybGQ=".to_string(), // "Hello World" in base64
        witness: None,
        witness_types: HashMap::new(),
        program_size: 11,
        source: None,
    };

    let decoded = output.decode_program().expect("Failed to decode");
    assert_eq!(decoded, b"Hello World");
}

#[test]
fn test_decode_witness_base64() {
    let output = CompiledOutput {
        cmr: "deadbeef".to_string(),
        program: "SGVsbG8=".to_string(),
        witness: Some("dGVzdCB3aXRuZXNz".to_string()), // "test witness" in base64
        witness_types: HashMap::new(),
        program_size: 5,
        source: None,
    };

    let decoded = output.decode_witness().expect("Failed to decode");
    assert_eq!(decoded, b"test witness");
}

#[test]
fn test_decode_empty_witness() {
    let output = CompiledOutput {
        cmr: "deadbeef".to_string(),
        program: "SGVsbG8=".to_string(),
        witness: None,
        witness_types: HashMap::new(),
        program_size: 5,
        source: None,
    };

    let decoded = output.decode_witness().expect("Failed to decode");
    assert!(decoded.is_empty());
}

#[test]
fn test_serialization_skips_none_fields() {
    let output = CompiledOutput {
        cmr: "deadbeef".to_string(),
        program: "SGVsbG8=".to_string(),
        witness: None,
        witness_types: HashMap::new(),
        program_size: 5,
        source: None,
    };

    let json = serde_json::to_string(&output).expect("Failed to serialize");

    // witness and source fields should not be in the JSON when None
    // Note: "witness_types" is always present, so we check for the exact key
    assert!(!json.contains("\"witness\":"));
    assert!(!json.contains("\"source\":"));
}

