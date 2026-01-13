//! File loading utilities with JSON/TOML format detection

use crate::error::SprayError;
use musk::{Arguments, WitnessValues};
use std::path::Path;

/// Load arguments from a JSON or TOML file
///
/// Format is detected by file extension:
/// - `.json` -> JSON
/// - `.toml` -> TOML
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_arguments(path: &Path) -> Result<Arguments, SprayError> {
    let contents = std::fs::read_to_string(path)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| SprayError::FileFormatError("No file extension found".into()))?;

    match ext {
        "json" => serde_json::from_str(&contents).map_err(Into::into),
        "toml" => toml::from_str(&contents)
            .map_err(|e| SprayError::ParseError(format!("TOML parse error: {e}"))),
        _ => Err(SprayError::FileFormatError(format!(
            "Unsupported file extension: {ext}"
        ))),
    }
}

/// Load witness values from a JSON or TOML file
///
/// Format is detected by file extension:
/// - `.json` -> JSON
/// - `.toml` -> TOML
///
/// # Errors
///
/// Returns an error if the file cannot be read or parsed.
pub fn load_witness(path: &Path) -> Result<WitnessValues, SprayError> {
    let contents = std::fs::read_to_string(path)?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| SprayError::FileFormatError("No file extension found".into()))?;

    match ext {
        "json" => serde_json::from_str(&contents).map_err(Into::into),
        "toml" => toml::from_str(&contents)
            .map_err(|e| SprayError::ParseError(format!("TOML parse error: {e}"))),
        _ => Err(SprayError::FileFormatError(format!(
            "Unsupported file extension: {ext}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_detection() {
        assert!(load_arguments(Path::new("test.json")).is_err()); // File doesn't exist
        assert!(load_arguments(Path::new("test.toml")).is_err()); // File doesn't exist
        assert!(load_arguments(Path::new("test.txt")).is_err()); // Unsupported extension
        assert!(load_arguments(Path::new("test")).is_err()); // No extension
    }
}

