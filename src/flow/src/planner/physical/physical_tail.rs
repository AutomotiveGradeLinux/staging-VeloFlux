// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::physical::BasePhysicalPlan;
use std::sync::Arc;

use super::PhysicalPlan;

/// Physical plan node for tail stage with multiple sinks.
/// This node serves as a pass-through collection point for multiple sink children,
/// maintaining the logical structure while allowing physical plan builder to process each sink.
#[derive(Debug, Clone)]
pub struct PhysicalTail {
    pub base: BasePhysicalPlan,
}

impl PhysicalTail {
    pub fn new(children: Vec<Arc<PhysicalPlan>>, index: i64) -> Self {
        Self {
            base: BasePhysicalPlan::new(children, index),
        }
    }
}