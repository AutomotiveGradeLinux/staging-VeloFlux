// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::planner::logical::{BaseLogicalPlan, LogicalPlan};
use sqlparser::ast::Expr;
use std::sync::Arc;

/// A computed (derived) field that is materialized into the tuple affiliate row.
#[derive(Debug, Clone)]
pub struct ComputeField {
    pub field_name: String,
    pub expr: Expr,
}

/// Logical operator that adds derived columns (affiliate fields) while preserving upstream
/// messages unchanged.
#[derive(Debug, Clone)]
pub struct Compute {
    pub base: BaseLogicalPlan,
    pub fields: Vec<ComputeField>,
}

impl Compute {
    pub fn new(fields: Vec<ComputeField>, children: Vec<Arc<LogicalPlan>>, index: i64) -> Self {
        let base = BaseLogicalPlan::new(children, index);
        Self { base, fields }
    }
}
