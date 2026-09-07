// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::BaseLogicalPlan;
use super::LogicalPlan;
use std::fmt;
use std::sync::Arc;

/// Logical plan node that represents the tail/end of a plan with multiple sinks.
/// This node acts as a collection point for multiple DataSinkPlan children,
/// providing a clean abstraction for multi-sink scenarios.
#[derive(Clone)]
pub struct TailPlan {
    pub base: BaseLogicalPlan,
}

impl TailPlan {
    /// Create a new tail plan with multiple sink children
    pub fn new(sink_children: Vec<Arc<LogicalPlan>>, index: i64) -> Self {
        Self {
            base: BaseLogicalPlan::new(sink_children, index),
        }
    }
}

impl fmt::Debug for TailPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TailPlan")
            .field("index", &self.base.index())
            .field("sink_count", &self.base.children.len())
            .finish()
    }
}
