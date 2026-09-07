// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::expr::ScalarExpr;
use crate::planner::physical::{BasePhysicalPlan, PhysicalPlan};
use sqlparser::ast::Expr;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct PhysicalOrderKey {
    pub original_expr: Expr,
    pub compiled_expr: ScalarExpr,
    pub asc: bool,
}

#[derive(Debug, Clone)]
pub struct PhysicalOrder {
    pub base: BasePhysicalPlan,
    pub keys: Vec<PhysicalOrderKey>,
}

impl PhysicalOrder {
    pub fn new(keys: Vec<PhysicalOrderKey>, children: Vec<Arc<PhysicalPlan>>, index: i64) -> Self {
        Self {
            base: BasePhysicalPlan::new(children, index),
            keys,
        }
    }
}
