// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

// Due to https://git.wiki.kernel.org/index.php/GitFaq#Why_does_Git_not_.22track.22_renames.3F
// we cannot rewire the git history of this file.
// check out the original commit history here:
// https://github.com/ChainSafe/forest/commits/main/forest/src/cli/mod.rs

mod auth_cmd;
mod chain_cmd;
mod config_cmd;
mod db_cmd;
mod fetch_params_cmd;
mod genesis_cmd;
mod mpool_cmd;
mod net_cmd;
mod send_cmd;
mod snapshot_cmd;
mod state_cmd;
mod sync_cmd;
mod wallet_cmd;

pub(super) use self::auth_cmd::AuthCommands;
pub(super) use self::chain_cmd::ChainCommands;
pub(super) use self::db_cmd::DBCommands;
pub(super) use self::fetch_params_cmd::FetchCommands;
pub(super) use self::genesis_cmd::GenesisCommands;
pub(super) use self::mpool_cmd::MpoolCommands;
pub(super) use self::net_cmd::NetCommands;
pub(super) use self::send_cmd::SendCommand;
pub(super) use self::snapshot_cmd::SnapshotCommands;
pub(super) use self::state_cmd::StateCommands;
pub(super) use self::sync_cmd::SyncCommands;
pub(super) use self::wallet_cmd::WalletCommands;
pub(crate) use forest_cli_shared::cli::{Config, FOREST_VERSION_STRING};

use crate::cli::config_cmd::ConfigCommands;
use cid::Cid;
use forest_blocks::tipset_json::TipsetJson;
use forest_cli_shared::cli::{to_size_string, CliOpts};
use http::StatusCode;
use jsonrpc_v2::Error as JsonRpcError;
use log::error;
use serde::Serialize;
use std::borrow::Cow;
use std::io::{self, Write};
use std::process;
use structopt::StructOpt;

/// CLI structure generated when interacting with Forest binary
#[derive(StructOpt)]
#[structopt(
    name = env!("CARGO_PKG_NAME"),
    version = FOREST_VERSION_STRING.as_str(),
    about = env!("CARGO_PKG_DESCRIPTION"),
    author = env!("CARGO_PKG_AUTHORS")
)]
pub struct Cli {
    #[structopt(flatten)]
    pub opts: CliOpts,
    #[structopt(subcommand)]
    pub cmd: Subcommand,
}

/// Forest binary sub-commands available.
#[derive(StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::VersionlessSubcommands)]
pub enum Subcommand {
    /// Download parameters for generating and verifying proofs for given size
    #[structopt(name = "fetch-params")]
    Fetch(FetchCommands),

    /// Interact with Filecoin blockchain
    Chain(ChainCommands),

    /// Manage RPC permissions
    Auth(AuthCommands),

    /// Work with blockchain genesis
    Genesis(GenesisCommands),

    /// Manage P2P network
    Net(NetCommands),

    /// Manage wallet
    Wallet(WalletCommands),

    /// Inspect or interact with the chain synchronizer
    Sync(SyncCommands),

    /// Interact with the message pool
    Mpool(MpoolCommands),

    /// Interact with and query Filecoin chain state
    State(StateCommands),

    /// Manage node configuration
    Config(ConfigCommands),

    /// Manage snapshots
    Snapshot(SnapshotCommands),

    /// Send funds between accounts
    Send(SendCommand),

    /// Database management
    DB(DBCommands),
}

/// Pretty-print a JSON-RPC error and exit
pub(super) fn handle_rpc_err(e: JsonRpcError) -> ! {
    let (code, message) = match e {
        JsonRpcError::Full { code, message, .. } => (code, Cow::from(message)),
        JsonRpcError::Provided { code, message } => (code, Cow::from(message)),
    };

    // we sometimes get negative errors, e.g. -32602 INVALID_PARAMS from `jsonrpc`.
    // It's a bit of a workaround to preserve partially the value but still try to fit into u16.
    let code = code.abs().clamp(0, u16::MAX.into()) as u16;

    match StatusCode::from_u16(code) {
        Ok(reason) => {
            error!("JSON RPC Error: Code: {reason} Message: {message}")
        }
        Err(_) => {
            error!("JSON RPC Error: Code: {code} Message: {message}")
        }
    };

    // fail-safe in case the `code` from `JsonRpcError` is zero. We still want to quit the process
    // with an error because, well, an error occurred if we are here.
    process::exit(code.max(1) as i32);
}

/// Format a vector to a prettified string
pub(super) fn format_vec_pretty(vec: Vec<String>) -> String {
    format!("[{}]", vec.join(", "))
}

/// Print an error message and exit the program with an error code
/// Used for handling high level errors such as invalid parameters
pub(super) fn cli_error_and_die(msg: impl AsRef<str>, code: i32) -> ! {
    error!("Error: {}", msg.as_ref());
    std::process::exit(code);
}

/// Prints a plain HTTP JSON-RPC response result
pub(super) fn print_rpc_res(res: Result<String, JsonRpcError>) {
    match res {
        Ok(obj) => println!("{}", &obj),
        Err(err) => handle_rpc_err(err),
    };
}

/// Prints a pretty HTTP JSON-RPC response result
pub(super) fn print_rpc_res_pretty<T: Serialize>(res: Result<T, JsonRpcError>) {
    match res {
        Ok(obj) => println!("{}", serde_json::to_string_pretty(&obj).unwrap()),
        Err(err) => handle_rpc_err(err),
    };
}

/// Prints a tipset from a HTTP JSON-RPC response result
pub(super) fn print_rpc_res_cids(res: Result<TipsetJson, JsonRpcError>) {
    match res {
        Ok(tipset) => println!(
            "{}",
            serde_json::to_string_pretty(
                &tipset
                    .0
                    .cids()
                    .iter()
                    .map(|cid: &Cid| cid.to_string())
                    .collect::<Vec<_>>()
            )
            .unwrap()
        ),
        Err(err) => handle_rpc_err(err),
    };
}

/// Prints a bytes HTTP JSON-RPC response result
pub(super) fn print_rpc_res_bytes(res: Result<Vec<u8>, JsonRpcError>) {
    match res {
        Ok(obj) => println!(
            "{}",
            String::from_utf8(obj)
                .map_err(|e| handle_rpc_err(e.into()))
                .unwrap()
        ),
        Err(err) => handle_rpc_err(err),
    };
}

/// Prints a string HTTP JSON-RPC response result to a buffered `stdout`
pub(super) fn print_stdout(out: String) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(out.as_bytes())
        .map_err(|e| handle_rpc_err(e.into()))
        .unwrap();

    handle
        .write("\n".as_bytes())
        .map_err(|e| handle_rpc_err(e.into()))
        .unwrap();
}

fn prompt_confirm() -> bool {
    println!("Do you want to continue? [y/n]");
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    let line = line.trim().to_lowercase();
    line == "y" || line == "yes"
}

#[cfg(test)]
mod test {
    use super::*;
    use fvm_shared::bigint::{BigInt, Zero};

    #[test]
    fn to_size_string_valid_input() {
        let cases = [
            (BigInt::zero(), "0 B"),
            (BigInt::from(1 << 10), "1024 B"),
            (BigInt::from((1 << 10) + 1), "1.00 KiB"),
            (BigInt::from((1 << 10) + 512), "1.50 KiB"),
            (BigInt::from(1 << 20), "1024.00 KiB"),
            (BigInt::from((1 << 20) + 1), "1.00 MiB"),
            (BigInt::from(1 << 29), "512.00 MiB"),
            (BigInt::from((1 << 30) + 1), "1.00 GiB"),
            (BigInt::from((1u64 << 40) + 1), "1.00 TiB"),
            (BigInt::from((1u64 << 50) + 1), "1.00 PiB"),
            // ZiB is 2^70, 288230376151711744 is 2^58
            (BigInt::from(u128::MAX), "288230376151711744.00 ZiB"),
        ];

        for (input, expected) in cases {
            assert_eq!(to_size_string(&input).unwrap(), expected.to_string());
        }
    }

    #[test]
    fn to_size_string_negative_input_should_fail() {
        assert!(to_size_string(&BigInt::from(-1i8)).is_err());
    }

    #[test]
    fn to_size_string_too_large_input_should_fail() {
        assert!(to_size_string(&(BigInt::from(u128::MAX) + 1)).is_err());
    }
}
