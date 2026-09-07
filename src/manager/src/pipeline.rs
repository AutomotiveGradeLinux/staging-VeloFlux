// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod context;
mod handlers;
mod remote;
mod runtime_failure;
pub(crate) mod scheduler;
mod spec;
pub(crate) mod state;
mod types;

pub use handlers::{
    collect_pipeline_stats_handler, create_pipeline_handler, delete_pipeline_handler,
    explain_pipeline_handler, get_pipeline_handler, list_pipelines, start_pipeline_handler,
    stop_pipeline_handler, upsert_pipeline_handler,
};
pub use state::AppState;
pub use types::CreatePipelineRequest;

pub(crate) use runtime_failure::{
    persist_generic_runtime_failure_marker, persist_start_failure_marker,
};
pub(crate) use spec::{
    build_pipeline_definition, referenced_streams_from_pipeline_sql, status_label,
    validate_create_request,
};
