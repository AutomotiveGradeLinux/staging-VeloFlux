// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::datatypes::DataType;
use crate::value::Value;

/// Boolean type
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BooleanType;

impl DataType for BooleanType {
    fn name(&self) -> String {
        "Boolean".to_string()
    }

    fn default_value(&self) -> Value {
        Value::Bool(false)
    }

    fn try_cast(&self, from: Value) -> Option<Value> {
        match from {
            Value::Null => Some(Value::Null),
            Value::Bool(v) => Some(Value::Bool(v)),
            _ => None,
        }
    }
}
