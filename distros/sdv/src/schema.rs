// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Schema definitions for CAN and GBF decoding. See `docs/schema.md` for details.

pub mod arxml;
pub mod busmirror;
pub mod dbc;
pub mod gbf;
pub mod ldf;
pub mod name_pattern;

pub use arxml::register_arxml_schema;
pub use busmirror::register_busmirror_schema;
pub use dbc::register_dbc_schema;
pub use gbf::register_gbf_schema;
