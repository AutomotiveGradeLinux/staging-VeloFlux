// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use http::StatusCode;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("{method} {path} transport error: {message}")]
    Transport {
        method: String,
        path: String,
        message: String,
    },

    #[error("{method} {path} url error: {source}")]
    Url {
        method: String,
        path: String,
        #[source]
        source: url::ParseError,
    },

    #[error("{method} {path} http {status}: {body}")]
    Http {
        method: String,
        path: String,
        status: StatusCode,
        body: String,
    },

    #[error("{method} {path} decode error: {source}")]
    Decode {
        method: String,
        path: String,
        #[source]
        source: serde_json::Error,
        raw_body: String,
    },
}

impl SdkError {
    pub fn transport(method: &str, path: &str, err: impl ToString) -> Self {
        Self::Transport {
            method: method.to_string(),
            path: path.to_string(),
            message: err.to_string(),
        }
    }

    pub fn url(method: &str, path: &str, err: url::ParseError) -> Self {
        Self::Url {
            method: method.to_string(),
            path: path.to_string(),
            source: err,
        }
    }

    pub fn http(method: &str, path: &str, status: StatusCode, body: String) -> Self {
        Self::Http {
            method: method.to_string(),
            path: path.to_string(),
            status,
            body,
        }
    }

    pub fn decode(method: &str, path: &str, err: serde_json::Error, raw_body: String) -> Self {
        Self::Decode {
            method: method.to_string(),
            path: path.to_string(),
            source: err,
            raw_body,
        }
    }
}
