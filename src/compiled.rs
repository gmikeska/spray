//! Compiled contract serialization format

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Serialized format for compiled Simplicity contracts
///
/// This format can be saved to JSON and later reloaded for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledOutput {
    /// Commitment Merkle Root (hex)
    pub cmr: String,
    /// Program bytes (base64 encoded)
    pub program: String,
    /// Witness bytes (base64 encoded), if witness was provided
    #[serde(skip_serializing_if = "Option::is_none")]
    pub witness: Option<String>,
    /// Witness types declared in the program
    pub witness_types: HashMap<String, String>,
    /// Program size in bytes
    pub program_size: usize,
    /// Source code (optional, for reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

impl CompiledOutput {
    /// Create a new compiled output from a musk `CompiledContract`
    #[must_use]
    pub fn from_compiled(compiled: &musk::CompiledContract, source: Option<String>) -> Self {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let program_bytes = compiled.inner().commit().to_vec_without_witness();
        let cmr = compiled.cmr();

        // Convert witness types to string map
        let witness_types = HashMap::new(); // TODO: Extract from compiled.inner().witness_types()

        Self {
            cmr: hex::encode(cmr.as_ref()),
            program: STANDARD.encode(&program_bytes),
            witness: None,
            witness_types,
            program_size: program_bytes.len(),
            source,
        }
    }

    /// Create from a satisfied program (includes witness)
    #[must_use]
    pub fn from_satisfied(
        satisfied: &musk::contract::SatisfiedContract,
        compiled: &musk::CompiledContract,
        source: Option<String>,
    ) -> Self {
        use base64::{engine::general_purpose::STANDARD, Engine};

        let (program_bytes, witness_bytes) = satisfied.encode();
        let cmr = compiled.cmr();

        let witness_types = HashMap::new(); // TODO: Extract from witness_types

        Self {
            cmr: hex::encode(cmr.as_ref()),
            program: STANDARD.encode(&program_bytes),
            witness: Some(STANDARD.encode(&witness_bytes)),
            witness_types,
            program_size: program_bytes.len(),
            source,
        }
    }

    /// Decode the program bytes from base64
    ///
    /// # Errors
    ///
    /// Returns an error if the base64 is invalid.
    pub fn decode_program(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD.decode(&self.program)
    }

    /// Decode the witness bytes from base64
    ///
    /// # Errors
    ///
    /// Returns an error if the base64 is invalid or witness is not present.
    pub fn decode_witness(&self) -> Result<Vec<u8>, base64::DecodeError> {
        use base64::{engine::general_purpose::STANDARD, Engine};

        self.witness
            .as_ref()
            .map_or_else(|| Ok(Vec::new()), |w| STANDARD.decode(w))
    }
}

// Add hex dependency
#[doc(hidden)]
mod hex {
    use std::fmt::Write;

    pub fn encode(bytes: &[u8]) -> String {
        bytes
            .iter()
            .fold(String::with_capacity(bytes.len() * 2), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            })
    }
}
