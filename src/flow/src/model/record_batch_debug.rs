// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::model::RecordBatch;

impl RecordBatch {
    pub fn debug_print(&self) {
        tracing::debug!(rows = self.num_rows(), "record batch");
        for (row_idx, row) in self.rows().iter().enumerate() {
            tracing::debug!(row_idx = row_idx, "record batch row");
            for ((source, column), value) in row.entries() {
                tracing::debug!(
                    source = source,
                    column = column,
                    value = ?value,
                    "record batch value"
                );
            }
        }
    }
}
