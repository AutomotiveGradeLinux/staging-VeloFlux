// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::physical::BasePhysicalPlan;
use crate::planner::sink::{SinkConnectorConfig, SinkRetryConfig};
use std::fmt;
use std::sync::Arc;

use super::PhysicalPlan;

/// Physical plan node for sink stage.
#[derive(Clone)]
pub struct PhysicalDataSink {
    pub base: BasePhysicalPlan,
    pub connector: PhysicalSinkConnector,
}

impl PhysicalDataSink {
    pub fn new(child: Arc<PhysicalPlan>, index: i64, connector: PhysicalSinkConnector) -> Self {
        Self {
            base: BasePhysicalPlan::new(vec![child], index),
            connector,
        }
    }
}

impl fmt::Debug for PhysicalDataSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PhysicalDataSink")
            .field("index", &self.base.index())
            .field("connector", &self.connector.sink_id)
            .finish()
    }
}

/// Declarative description of a sink connector bound to an encoder node.
///
/// Note: connector_id has been removed as it's not needed for processor identification.
/// Processor IDs are now generated using only sink_id and physical_plan_name.
#[derive(Clone)]
pub struct PhysicalSinkConnector {
    pub sink_id: String,
    pub forward_to_result: bool,
    pub connector: SinkConnectorConfig,
    pub encoder_plan_index: Option<i64>,
    pub retry: SinkRetryConfig,
}

impl PhysicalSinkConnector {
    pub fn new(
        sink_id: String,
        forward_to_result: bool,
        connector: SinkConnectorConfig,
        encoder_plan_index: Option<i64>,
        retry: SinkRetryConfig,
    ) -> Self {
        Self {
            sink_id,
            forward_to_result,
            connector,
            encoder_plan_index,
            retry,
        }
    }
}
