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

pub mod client;
pub mod config;
pub mod error;
pub mod tls;
pub mod transport;
pub mod types;

pub use client::ManagerClient;
pub use config::ClientConfig;
pub use error::SdkError;
pub use tls::install_default_crypto_provider;
pub use types::{
    PipelineCreateRequest, StopMode, StopOptions, StreamCreateRequest, StreamUpsertRequest,
};
