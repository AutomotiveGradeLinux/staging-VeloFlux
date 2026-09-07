// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use parser::StatefulCallSpec;
use std::sync::Arc;

use crate::planner::logical::{BaseLogicalPlan, LogicalPlan};

#[derive(Debug, Clone)]
pub struct LogicalStatefulCall {
    pub output_column: String,
    pub spec: StatefulCallSpec,
}

#[derive(Debug, Clone)]
pub struct StatefulFunctionPlan {
    pub base: BaseLogicalPlan,
    pub calls: Vec<LogicalStatefulCall>,
}

impl StatefulFunctionPlan {
    pub fn new(
        calls: Vec<LogicalStatefulCall>,
        children: Vec<Arc<LogicalPlan>>,
        index: i64,
    ) -> Self {
        Self {
            base: BaseLogicalPlan::new(children, index),
            calls,
        }
    }
}
