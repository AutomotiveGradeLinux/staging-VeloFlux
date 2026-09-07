// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::unreachable, clippy::panic)
)]
#![forbid(unsafe_code)]

pub mod abi;
pub mod engine;
pub mod func;
pub mod metadata;

pub use abi::AbiError;
pub use engine::{WasmEngine, WasmError};
pub use func::WasmUdf;
pub use metadata::{UdfArg, UdfMetadata};
