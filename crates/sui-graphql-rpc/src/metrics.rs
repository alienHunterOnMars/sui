// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use prometheus::{register_histogram_with_registry, Histogram, Registry};

#[derive(Clone, Debug)]
pub struct RequestMetrics {
    pub(crate) num_nodes: Histogram,
    pub(crate) query_depth: Histogram,
}

// TODO: finetune buckets as we learn more about the distribution of queries
const NUM_NODES_BUCKETS: &[f64] = &[
    1., 2., 4., 8., 12., 16., 24., 32., 48., 64., 96., 128., 256., 512., 1024.,
];
const QUERY_DEPTH_BUCKETS: &[f64] = &[
    1., 2., 4., 8., 12., 16., 24., 32., 48., 64., 96., 128., 256., 512., 1024.,
];

impl RequestMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            num_nodes: register_histogram_with_registry!(
                "num_nodes",
                "Number of nodes in the query",
                NUM_NODES_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
            query_depth: register_histogram_with_registry!(
                "query_depth",
                "Depth of the query",
                QUERY_DEPTH_BUCKETS.to_vec(),
                registry,
            )
            .unwrap(),
        }
    }
}
