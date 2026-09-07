// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::physical::{BasePhysicalPlan, PhysicalPlan, WatermarkConfig};
use std::sync::Arc;

/// Processing-time watermark physical node (ticker-based).
///
/// Note: this type is introduced as part of the PhysicalPlan watermark split (step 6).
#[derive(Debug, Clone)]
pub struct PhysicalProcessTimeWatermark {
    pub base: BasePhysicalPlan,
    pub config: WatermarkConfig,
}

impl PhysicalProcessTimeWatermark {
    pub fn new(config: WatermarkConfig, children: Vec<Arc<PhysicalPlan>>, index: i64) -> Self {
        let base = BasePhysicalPlan::new(children, index);
        Self { base, config }
    }
}
