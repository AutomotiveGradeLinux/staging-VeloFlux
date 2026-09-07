// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::time::Duration;
use url::Url;

#[derive(Clone, Debug)]
pub struct ClientConfig {
    pub base_url: Url,
    pub timeout: Duration,
}

impl Default for ClientConfig {
    // SAFETY: parsing a known-good string literal constant; can never fail
    #[allow(clippy::expect_used)]
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080"
                .parse()
                .expect("default base_url must be valid"),
            timeout: Duration::from_secs(30),
        }
    }
}

impl ClientConfig {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            ..Default::default()
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}
