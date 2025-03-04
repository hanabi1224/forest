// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use cid::Error as CidErr;
use forest_blocks::Error as BlkErr;
use forest_db::Error as DbErr;
use forest_encoding::error::Error as SerdeErr;
use forest_legacy_ipld_amt::Error as AmtErr;
use fvm_ipld_encoding::Error as EncErr;
use thiserror::Error;

/// Chain error
#[derive(Debug, Error)]
pub enum Error {
    /// Key was not found
    #[error("Invalid tipset: {0}")]
    UndefinedKey(String),
    /// Tipset contains no blocks
    #[error("No blocks for tipset")]
    NoBlocks,
    /// Key not found in database
    #[error("{0} not found")]
    NotFound(String),
    /// Error originating from key-value store
    #[error(transparent)]
    DB(#[from] DbErr),
    /// Error originating constructing blockchain structures
    #[error(transparent)]
    Blockchain(#[from] BlkErr),
    /// Error originating from encoding arbitrary data
    #[error("{0}")]
    Encoding(String),
    /// Error originating from Cid creation
    #[error(transparent)]
    Cid(#[from] CidErr),
    /// Amt error
    #[error("State error: {0}")]
    State(String),
    /// Other chain error
    #[error("{0}")]
    Other(String),
}

impl From<EncErr> for Error {
    fn from(e: EncErr) -> Error {
        Error::Encoding(e.to_string())
    }
}

impl From<SerdeErr> for Error {
    fn from(e: SerdeErr) -> Error {
        Error::Encoding(e.to_string())
    }
}

impl From<AmtErr> for Error {
    fn from(e: AmtErr) -> Error {
        Error::State(e.to_string())
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::Other(e)
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Other(e.to_string())
    }
}

impl<T> From<flume::SendError<T>> for Error {
    fn from(e: flume::SendError<T>) -> Self {
        Error::Other(e.to_string())
    }
}
