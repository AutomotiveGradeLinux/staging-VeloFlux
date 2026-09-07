// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod collection;
pub mod record_batch;
#[cfg(debug_assertions)]
mod record_batch_debug;
pub mod record_batch_impl;
pub mod tuple;

pub use collection::{Collection, CollectionError, CollectionMetadata, Column, WindowMetadata};
pub use record_batch::{
    batch_from_columns, batch_from_columns_simple, rows_from_columns, rows_from_columns_simple,
    RecordBatch,
};
pub use tuple::{AffiliateRow, Message, Tuple};
