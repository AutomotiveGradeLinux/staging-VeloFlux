// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Visitor pattern implementations for SQL AST traversal

pub mod aggregate_visitor;
pub mod table_visitor;

pub use aggregate_visitor::{
    AggregateVisitor, contains_aggregates_with_visitor, extract_aggregates_with_visitor,
};
pub use table_visitor::TableInfoVisitor;
// SourceInfo is defined in select_stmt, not in table_visitor
pub use crate::select_stmt::SourceInfo;
