// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::physical::{BasePhysicalPlan, PhysicalPlan, WatermarkConfig};
use std::sync::Arc;
use std::time::Duration;

/// Event-time watermark physical node (data-driven).
#[derive(Debug, Clone)]
pub struct PhysicalEventtimeWatermark {
    pub base: BasePhysicalPlan,
    pub late_tolerance: Duration,
    pub window_config: Option<WatermarkConfig>,
}

impl PhysicalEventtimeWatermark {
    pub fn new(
        late_tolerance: Duration,
        window_config: Option<WatermarkConfig>,
        children: Vec<Arc<PhysicalPlan>>,
        index: i64,
    ) -> Self {
        let base = BasePhysicalPlan::new(children, index);
        Self {
            base,
            late_tolerance,
            window_config,
        }
    }
}
