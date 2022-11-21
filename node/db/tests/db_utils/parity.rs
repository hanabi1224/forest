// Copyright 2019-2022 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use std::ops::Deref;

use forest_db::parity_db::{ParityDb, ParityDbConfig};

/// Temporary, self-cleaning ParityDB
pub struct TempParityDB {
    db: ParityDb,
    _dir: tempfile::TempDir, // kept for cleaning up during Drop
}

impl TempParityDB {
    /// Creates a new DB in a temporary path that gets wiped out when the variable
    /// gets out of scope.
    pub fn new() -> TempParityDB {
        let dir = tempfile::Builder::new()
            .tempdir()
            .expect("Failed to create temporary path for db.");
        let path = dir.path().join("paritydb");

        let config = ParityDbConfig::from_path(&path);

        TempParityDB {
            db: ParityDb::open(&config).unwrap(),
            _dir: dir,
        }
    }
}

impl Deref for TempParityDB {
    type Target = ParityDb;

    fn deref(&self) -> &Self::Target {
        &self.db
    }
}
