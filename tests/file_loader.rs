//! Unit tests for file loading utilities

use spray::error::SprayError;
use spray::file_loader::{load_arguments, load_witness};
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

fn create_temp_file(extension: &str, contents: &str) -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .suffix(extension)
        .tempfile()
        .expect("Failed to create temp file");
    file.write_all(contents.as_bytes())
        .expect("Failed to write temp file");
    file
}

#[test]
fn test_load_arguments_json() {
    let file = create_temp_file(".json", "{}");
    let result = load_arguments(file.path());
    assert!(result.is_ok(), "Should parse empty JSON arguments");
}

#[test]
fn test_load_arguments_toml() {
    let file = create_temp_file(".toml", "");
    let result = load_arguments(file.path());
    assert!(result.is_ok(), "Should parse empty TOML arguments");
}

#[test]
fn test_load_witness_json() {
    let file = create_temp_file(".json", "{}");
    let result = load_witness(file.path());
    assert!(result.is_ok(), "Should parse empty JSON witness");
}

#[test]
fn test_load_witness_toml() {
    let file = create_temp_file(".toml", "");
    let result = load_witness(file.path());
    assert!(result.is_ok(), "Should parse empty TOML witness");
}

#[test]
fn test_invalid_extension_error() {
    let file = create_temp_file(".txt", "some content");
    let result = load_arguments(file.path());

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        SprayError::FileFormatError(msg) => {
            assert!(msg.contains("txt"), "Error should mention the extension");
        }
        _ => panic!("Expected FileFormatError, got {:?}", err),
    }
}

#[test]
fn test_missing_extension_error() {
    // Create a path without extension
    let result = load_arguments(Path::new("/tmp/no_extension_file_that_does_not_exist"));

    assert!(result.is_err());
    // Could be either FileFormatError (no extension) or IoError (file not found)
    // depending on which check happens first
}

#[test]
fn test_nonexistent_file_error() {
    let result = load_arguments(Path::new("/nonexistent/path/file.json"));

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        SprayError::IoError(_) => {} // Expected
        _ => panic!("Expected IoError, got {:?}", err),
    }
}

#[test]
fn test_invalid_json_error() {
    let file = create_temp_file(".json", "{ invalid json }");
    let result = load_arguments(file.path());

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        SprayError::JsonError(_) => {} // Expected
        _ => panic!("Expected JsonError, got {:?}", err),
    }
}

#[test]
fn test_invalid_toml_error() {
    let file = create_temp_file(".toml", "[[invalid toml");
    let result = load_arguments(file.path());

    assert!(result.is_err());
    let err = result.unwrap_err();
    match err {
        SprayError::ParseError(msg) => {
            assert!(msg.contains("TOML"), "Error should mention TOML");
        }
        _ => panic!("Expected ParseError, got {:?}", err),
    }
}
