// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::logical::{BaseLogicalPlan, LogicalPlan};
use sqlparser::ast::Expr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct ProjectField {
    pub field_name: String,
    pub expr: Expr,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub base: BaseLogicalPlan,
    pub fields: Vec<ProjectField>,
}

impl Project {
    pub fn new(fields: Vec<ProjectField>, children: Vec<Arc<LogicalPlan>>, index: i64) -> Self {
        let base = BaseLogicalPlan::new(children, index);
        Self { base, fields }
    }
}
