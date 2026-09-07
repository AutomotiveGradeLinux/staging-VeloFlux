// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::expect_used)] // prometheus metric registration — Lazy::new closures run once at startup

use prometheus::core::Collector;
use prometheus::{HistogramOpts, Opts};

pub(crate) const NAMESPACE: &str = "veloflux";

pub(crate) const FLOW_INSTANCE_LABEL: &str = "flow_instance";
pub(crate) const PIPELINE_ID_LABEL: &str = "pipeline_id";
pub(crate) const PROCESSOR_ID_LABEL: &str = "processor_id";
pub(crate) const CONNECTOR_LABEL: &str = "connector";
pub(crate) const KEY_LABEL: &str = "key";
pub(crate) const KIND_LABEL: &str = "kind";
pub(crate) const METRIC_LABEL: &str = "metric";

pub(crate) fn opts(subsystem: &str, name: &str, help: &str) -> Opts {
    Opts::new(name, help)
        .namespace(NAMESPACE)
        .subsystem(subsystem)
}

pub(crate) fn histogram_opts(subsystem: &str, name: &str, help: &str) -> HistogramOpts {
    HistogramOpts::new(name, help)
        .namespace(NAMESPACE)
        .subsystem(subsystem)
}

pub(crate) fn register_collector<T>(collector: T) -> T
where
    T: Collector + Clone + 'static,
{
    prometheus::register(Box::new(collector.clone())).expect("register prometheus collector");
    collector
}
