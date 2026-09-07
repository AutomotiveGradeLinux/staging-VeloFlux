// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::datatypes::DataType;
use crate::value::Value;

/// String type
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StringType;

impl DataType for StringType {
    fn name(&self) -> String {
        "String".to_string()
    }

    fn default_value(&self) -> Value {
        Value::String(String::new())
    }

    fn try_cast(&self, from: Value) -> Option<Value> {
        match from {
            Value::Null => Some(Value::Null),
            Value::String(s) => Some(Value::String(s)),
            Value::Timestamp(ts) => ts.to_rfc3339_utc().map(Value::String),
            _ => None,
        }
    }
}
