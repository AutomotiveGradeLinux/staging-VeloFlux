// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};

/// Metadata extracted from a WASM UDF module's `udf_metadata` export.
///
/// Every WASM UDF must export `udf_metadata_ptr() -> i32` and `udf_metadata_len() -> i32`
/// pointing to a JSON blob matching this struct.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UdfMetadata {
    /// Canonical function name (must match the externally assigned name).
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// Additional SQL-visible aliases for this function.
    #[serde(default)]
    pub aliases: Vec<String>,
    /// Argument descriptors in call order.
    #[serde(default)]
    pub args: Vec<UdfArg>,
    /// Expected return type.
    #[serde(default)]
    pub return_type: String,
    /// Whether this UDF supports batch evaluation.
    #[serde(default)]
    pub batch_mode: bool,
}

/// Descriptor for a single argument.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UdfArg {
    /// Argument name (for documentation).
    pub name: String,
    /// Expected type: "int", "float", "string", "bool", "timestamp", "object", "array", "any".
    #[serde(rename = "type")]
    pub arg_type: String,
}
