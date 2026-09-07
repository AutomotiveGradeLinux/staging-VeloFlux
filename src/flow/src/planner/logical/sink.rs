// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::BaseLogicalPlan;
use super::LogicalPlan;
use crate::planner::sink::PipelineSink;
use std::fmt;
use std::sync::Arc;

/// Logical plan node that represents a single sink definition.
#[derive(Clone)]
pub struct DataSinkPlan {
    pub base: BaseLogicalPlan,
    pub sink: PipelineSink,
}

impl DataSinkPlan {
    pub fn new(child: Arc<LogicalPlan>, index: i64, sink: PipelineSink) -> Self {
        Self {
            base: BaseLogicalPlan::new(vec![child], index),
            sink,
        }
    }
}

impl fmt::Debug for DataSinkPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataSinkPlan")
            .field("index", &self.base.index())
            .field("sink_id", &self.sink.sink_id)
            .finish()
    }
}
