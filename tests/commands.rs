//! Unit tests for command helpers

use spray::commands::compile::OutputFormat;

#[test]
fn test_output_format_parse_json() {
    let format = OutputFormat::parse("json");
    assert_eq!(format, Some(OutputFormat::Json));
}

#[test]
fn test_output_format_parse_base64() {
    let format = OutputFormat::parse("base64");
    assert_eq!(format, Some(OutputFormat::Base64));
}

#[test]
fn test_output_format_parse_hex() {
    let format = OutputFormat::parse("hex");
    assert_eq!(format, Some(OutputFormat::Hex));
}

#[test]
fn test_output_format_parse_invalid() {
    let format = OutputFormat::parse("invalid");
    assert_eq!(format, None);
}

#[test]
fn test_output_format_parse_case_sensitive() {
    // Should be case-sensitive (lowercase only)
    assert_eq!(OutputFormat::parse("JSON"), None);
    assert_eq!(OutputFormat::parse("Json"), None);
    assert_eq!(OutputFormat::parse("BASE64"), None);
    assert_eq!(OutputFormat::parse("HEX"), None);
}

#[test]
fn test_output_format_equality() {
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
    assert_eq!(OutputFormat::Base64, OutputFormat::Base64);
    assert_eq!(OutputFormat::Hex, OutputFormat::Hex);
    
    assert_ne!(OutputFormat::Json, OutputFormat::Base64);
    assert_ne!(OutputFormat::Json, OutputFormat::Hex);
    assert_ne!(OutputFormat::Base64, OutputFormat::Hex);
}

// Tests for parse_utxo_ref
use spray::commands::parse_utxo_ref;

#[test]
fn test_parse_utxo_ref_valid() {
    let valid_ref = "0000000000000000000000000000000000000000000000000000000000000000:0";
    let result = parse_utxo_ref(valid_ref);
    
    assert!(result.is_ok(), "Valid UTXO ref should parse");
    let (txid, vout) = result.unwrap();
    assert_eq!(vout, 0);
    assert_eq!(txid.to_string(), "0000000000000000000000000000000000000000000000000000000000000000");
}

#[test]
fn test_parse_utxo_ref_valid_nonzero_vout() {
    let valid_ref = "0000000000000000000000000000000000000000000000000000000000000001:42";
    let result = parse_utxo_ref(valid_ref);
    
    assert!(result.is_ok());
    let (_, vout) = result.unwrap();
    assert_eq!(vout, 42);
}

#[test]
fn test_parse_utxo_ref_invalid_format_no_colon() {
    let invalid_ref = "0000000000000000000000000000000000000000000000000000000000000000";
    let result = parse_utxo_ref(invalid_ref);
    
    assert!(result.is_err(), "Should fail without colon separator");
}

#[test]
fn test_parse_utxo_ref_invalid_format_wrong_separator() {
    let invalid_ref = "0000000000000000000000000000000000000000000000000000000000000000-0";
    let result = parse_utxo_ref(invalid_ref);
    
    assert!(result.is_err(), "Should fail with wrong separator");
}

#[test]
fn test_parse_utxo_ref_invalid_txid() {
    let invalid_ref = "invalid:0";
    let result = parse_utxo_ref(invalid_ref);
    
    assert!(result.is_err(), "Should fail with invalid txid");
}

#[test]
fn test_parse_utxo_ref_invalid_vout() {
    let invalid_ref = "0000000000000000000000000000000000000000000000000000000000000000:abc";
    let result = parse_utxo_ref(invalid_ref);
    
    assert!(result.is_err(), "Should fail with non-numeric vout");
}

#[test]
fn test_parse_utxo_ref_too_many_parts() {
    let invalid_ref = "0000000000000000000000000000000000000000000000000000000000000000:0:extra";
    let result = parse_utxo_ref(invalid_ref);
    
    assert!(result.is_err(), "Should fail with too many parts");
}

