// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::physical::output_layout::OutputLayout;
use crate::planner::physical::{BasePhysicalPlan, PhysicalPlan};
use std::sync::Arc;

/// Materialize an arbitrary incoming `Collection` into a stable sink output layout.
///
/// The node reshapes each tuple to `1 message + 0 affiliate` with keys ordered by the planned
/// output schema, filling missing columns with NULL.
#[derive(Debug, Clone)]
pub struct PhysicalMemoryCollectionMaterialize {
    pub base: BasePhysicalPlan,
    pub output_layout: OutputLayout,
}

impl PhysicalMemoryCollectionMaterialize {
    pub fn new(output_layout: OutputLayout, child: Arc<PhysicalPlan>, index: i64) -> Self {
        Self {
            base: BasePhysicalPlan::new(vec![child], index),
            output_layout,
        }
    }
}
