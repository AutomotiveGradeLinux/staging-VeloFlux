// Copyright 2026 EMQ Technologies Co., Ltd.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod avg;
mod count;
mod deduplicate;
mod extrema;
mod last_row;
mod median;
mod merge_agg;
mod ndv;
mod registry;
mod sum;
mod variance;

pub use avg::{avg_function_def, AvgFunction};
pub use count::{count_function_def, CountFunction};
pub use deduplicate::{deduplicate_function_def, DeduplicateFunction};
pub use extrema::{max_function_def, min_function_def, MaxFunction, MinFunction};
pub use last_row::last_row_function_def;
pub use last_row::LastRowFunction;
pub use median::{median_function_def, MedianFunction};
pub use merge_agg::{merge_agg_function_def, MergeAggFunction};
pub use ndv::ndv_function_def;
pub use ndv::NdvFunction;
pub use registry::builtin_aggregation_defs;
pub use registry::{
    AggregateAccumulator, AggregateFunction, AggregateFunctionRegistry, AggregateUpdate,
};
pub use sum::sum_function_def;
pub use sum::SumFunction;
pub use variance::{
    stddev_function_def, stddevs_function_def, var_function_def, vars_function_def, StddevFunction,
    StddevsFunction, VarFunction, VarsFunction,
};
