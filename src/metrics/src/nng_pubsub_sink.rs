// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![allow(clippy::expect_used)] // prometheus metric registration — Lazy::new closures run once at startup

use once_cell::sync::Lazy;
use prometheus::IntCounterVec;

use crate::common::{opts, register_collector, CONNECTOR_LABEL, FLOW_INSTANCE_LABEL};

static NNG_PUBSUB_SINK_MESSAGES_IN_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_collector(
        IntCounterVec::new(
            opts(
                "nng_pubsub_sink",
                "messages_in_total",
                "Number of messages received by NNG pubsub sink connectors",
            ),
            &[FLOW_INSTANCE_LABEL, CONNECTOR_LABEL],
        )
        .expect("create nng pubsub sink messages_in counter vec"),
    )
});

static NNG_PUBSUB_SINK_MESSAGES_OUT_TOTAL: Lazy<IntCounterVec> = Lazy::new(|| {
    register_collector(
        IntCounterVec::new(
            opts(
                "nng_pubsub_sink",
                "messages_out_total",
                "Number of messages successfully published by NNG pubsub sink connectors",
            ),
            &[FLOW_INSTANCE_LABEL, CONNECTOR_LABEL],
        )
        .expect("create nng pubsub sink messages_out counter vec"),
    )
});

pub fn messages_in_total() -> &'static IntCounterVec {
    &NNG_PUBSUB_SINK_MESSAGES_IN_TOTAL
}

pub fn messages_out_total() -> &'static IntCounterVec {
    &NNG_PUBSUB_SINK_MESSAGES_OUT_TOTAL
}
