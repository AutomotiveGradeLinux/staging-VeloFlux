// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::datatypes::DataType;
use crate::value::Value;

/// 32-bit floating point number type
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Float32Type;

impl DataType for Float32Type {
    fn name(&self) -> String {
        "Float32".to_string()
    }

    fn default_value(&self) -> Value {
        Value::Float32(0.0)
    }

    fn try_cast(&self, from: Value) -> Option<Value> {
        match from {
            Value::Null => Some(Value::Null),
            Value::Float32(v) => Some(Value::Float32(v)),
            Value::Float64(v) => Some(Value::Float32(v as f32)),
            Value::Int8(v) => Some(Value::Float32(v as f32)),
            Value::Int16(v) => Some(Value::Float32(v as f32)),
            Value::Int32(v) => Some(Value::Float32(v as f32)),
            Value::Int64(v) => Some(Value::Float32(v as f32)),
            Value::Uint8(v) => Some(Value::Float32(v as f32)),
            Value::Uint16(v) => Some(Value::Float32(v as f32)),
            Value::Uint32(v) => Some(Value::Float32(v as f32)),
            Value::Uint64(v) => Some(Value::Float32(v as f32)),
            _ => None,
        }
    }
}
