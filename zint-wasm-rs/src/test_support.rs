//! Helpers shared by the unit tests across this crate.
//!
//! Options reach this library as CBOR, encoded by Typst's `cbor.encode`, so the
//! deserialization tests go through that same format rather than a friendlier
//! one. The hand written visitors and `#[serde(flatten)]` both behave
//! differently depending on what the format tells `serde` about the data, so a
//! test against a different format would not prove much about the real input.

use serde::de::DeserializeOwned;

/// Deserializes a CBOR value the way the Typst plugin does: by encoding it and
/// reading it back.
///
/// The error is a [`String`] because [`serde::de::Error::custom`], which every
/// visitor in this crate reports through, erases the concrete error type.
pub fn from_cbor<T: DeserializeOwned>(value: ciborium::Value) -> Result<T, String> {
    let mut encoded = Vec::new();
    ciborium::into_writer(&value, &mut encoded).expect("CBOR value is encodable");
    ciborium::from_reader(encoded.as_slice()).map_err(|error| error.to_string())
}
