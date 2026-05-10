//! TOON format encoding wrapper
//!
//! Thin wrapper around the `toon` crate for TOON formatting.

/// Encode a JSON value into TOON format
pub fn encode(value: &serde_json::Value) -> String {
    toon::encode(value, None)
}
