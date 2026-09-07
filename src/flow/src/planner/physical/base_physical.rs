// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::physical::PhysicalPlan;
use std::sync::Arc;

/// Base struct for physical plans containing common fields
///
/// This provides the basic structure that all physical plans can build upon,
/// including children references and plan identification.
#[derive(Debug, Clone)]
pub struct BasePhysicalPlan {
    /// Unique identifier for this physical plan node
    pub index: i64,

    /// Child physical plans that this plan depends on
    pub children: Vec<Arc<PhysicalPlan>>,
}

impl BasePhysicalPlan {
    /// Create a new BasePhysicalPlan
    pub fn new(children: Vec<Arc<PhysicalPlan>>, index: i64) -> Self {
        Self { children, index }
    }

    /// Create a new BasePhysicalPlan with no children (leaf node)
    pub fn new_leaf(index: i64) -> Self {
        Self {
            children: Vec::new(),
            index,
        }
    }

    pub fn children(&self) -> &[Arc<PhysicalPlan>] {
        &self.children
    }

    pub fn index(&self) -> i64 {
        self.index
    }
}
