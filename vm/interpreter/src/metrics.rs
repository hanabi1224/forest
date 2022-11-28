// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use lazy_static::lazy_static;
use prometheus::{core::Opts, HistogramOpts, HistogramVec};

lazy_static! {
    pub static ref APPLY_BLOCK_MESSAGES_TASKS_TIME: Box<HistogramVec> = {
        let apply_block_msgs_tasks_time = Box::new(
            HistogramVec::new(
                HistogramOpts {
                    common_opts: Opts::new(
                        "apply_block_msgs_tasks_time",
                        "Duration of subroutines inside apply_block_messages",
                    ),
                    buckets: vec![],
                },
                &["type"],
            )
            .expect("Defining the apply_block_msgs_tasks_time metric must succeed"),
        );
        prometheus::default_registry().register(apply_block_msgs_tasks_time.clone()).expect(
            "Registering the apply_block_msgs_tasks_time metric with the metrics registry must succeed",
        );
        apply_block_msgs_tasks_time
    };
}

pub mod values {
    pub const APPLY_IMPLICIT: &str = "apply_implicit";
    pub const RUN_CRON: &str = "run_cron";
}
