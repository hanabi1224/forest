// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

mod cli;
mod subcommand;

use cli::{cli_error_and_die, Cli};

use async_std::task;
use forest_cli_shared::logger;
use lazy_static::lazy_static;
use structopt::StructOpt;
use tempfile::{Builder, TempPath};

lazy_static! {
    pub static ref IPC_PATH: TempPath = Builder::new()
        .prefix("forest-ipc")
        .tempfile()
        .expect("tempfile must succeed")
        .into_temp_path();
}

fn main() {
    // Capture Cli inputs
    let Cli { opts, cmd } = Cli::from_args();

    // Run forest as a daemon if no other subcommands are used. Otherwise, run the subcommand.
    match opts.to_config() {
        Ok(cfg) => {
            logger::setup_logger(&cfg.log);
            task::block_on(subcommand::process(cmd, cfg));
        }
        Err(e) => {
            cli_error_and_die(format!("Error parsing config. Error was: {e}"), 1);
        }
    };
}