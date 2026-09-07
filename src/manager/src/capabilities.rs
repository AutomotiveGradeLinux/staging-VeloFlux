// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use axum::{Json, response::IntoResponse};

pub async fn get_syntax_capabilities_handler() -> impl IntoResponse {
    Json(parser::syntax_capabilities_owned())
}
