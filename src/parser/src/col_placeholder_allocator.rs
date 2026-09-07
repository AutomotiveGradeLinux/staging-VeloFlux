// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[derive(Debug, Default)]
pub struct ColPlaceholderAllocator {
    next_index: usize,
}

impl ColPlaceholderAllocator {
    pub fn new() -> Self {
        Self { next_index: 0 }
    }

    pub fn allocate(&mut self) -> String {
        self.next_index += 1;
        format!("col_{}", self.next_index)
    }
}
