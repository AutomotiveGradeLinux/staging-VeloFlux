// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::fmt;
use std::net::SocketAddr;

#[derive(Debug, Default, Clone, Copy)]
pub struct PrometheusBuilder {
    http_listener: Option<SocketAddr>,
}

impl PrometheusBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_http_listener(mut self, addr: SocketAddr) -> Self {
        self.http_listener = Some(addr);
        self
    }

    pub fn install(self) -> Result<(), InstallError> {
        let _ = self.http_listener;
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct InstallError;

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("prometheus metrics exporter shim install failed")
    }
}

impl std::error::Error for InstallError {}
