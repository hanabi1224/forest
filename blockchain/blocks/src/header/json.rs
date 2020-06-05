// Copyright 2020 ChainSafe Systems
// SPDX-License-Identifier: Apache-2.0, MIT

use super::*;
use crate::{ticket, tipset::tipset_keys_json};
use beacon::beacon_entries;
use crypto::{election_proof, signature};
use fil_types::sector::post;
use serde::{de, Deserialize, Serialize};

// Wrapper for serializing and deserializing a BlockHeader from JSON.
#[derive(Deserialize, Serialize)]
#[serde(transparent)]
pub struct BlockHeaderJson(#[serde(with = "self")] pub BlockHeader);

/// Wrapper for serializing a BlockHeader reference to JSON.
#[derive(Serialize)]
#[serde(transparent)]
pub struct BlockHeaderJsonRef<'a>(#[serde(with = "self")] pub &'a BlockHeader);

pub fn serialize<S>(m: &BlockHeader, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    #[serde(rename_all = "PascalCase")]
    struct BlockHeaderSer<'a> {
        miner: String,
        #[serde(with = "ticket::json")]
        ticket: &'a Ticket,
        #[serde(with = "election_proof::json::opt")]
        election_proof: &'a Option<ElectionProof>,
        #[serde(with = "beacon_entries::json::vec")]
        beacon_entries: &'a [BeaconEntry],
        #[serde(rename = "WinPoStProof", with = "post::json::vec")]
        win_post_proof: &'a [PoStProof],
        #[serde(rename = "Parents", with = "tipset_keys_json")]
        parents: &'a TipsetKeys,
        #[serde(rename = "ParentWeight")]
        weight: String,
        height: &'a u64,
        #[serde(rename = "ParentStateRoot", with = "cid::json")]
        state_root: &'a Cid,
        #[serde(rename = "ParentMessageReceipts", with = "cid::json")]
        message_receipts: &'a Cid,
        #[serde(with = "cid::json")]
        messages: &'a Cid,
        #[serde(rename = "BLSAggregate", with = "signature::json::opt")]
        bls_aggregate: &'a Option<Signature>,
        timestamp: &'a u64,
        #[serde(rename = "BlockSig", with = "signature::json::opt")]
        signature: &'a Option<Signature>,
        #[serde(rename = "ForkSignaling")]
        fork_signal: &'a u64,
    }

    BlockHeaderSer {
        miner: m.miner_address.to_string(),
        ticket: &m.ticket,
        election_proof: &m.election_proof,
        win_post_proof: &m.win_post_proof,
        parents: &m.parents,
        weight: m.weight.to_string(),
        height: &m.epoch,
        state_root: &m.state_root,
        message_receipts: &m.message_receipts,
        messages: &m.messages,
        bls_aggregate: &m.bls_aggregate,
        timestamp: &m.timestamp,
        beacon_entries: &m.beacon_entries,
        signature: &m.signature,
        fork_signal: &m.fork_signal,
    }
    .serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<BlockHeader, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(rename_all = "PascalCase")]
    struct BlockHeaderDe {
        miner: String,
        #[serde(with = "ticket::json")]
        ticket: Ticket,
        #[serde(default, with = "election_proof::json::opt")]
        election_proof: Option<ElectionProof>,
        #[serde(default, with = "beacon_entries::json::vec")]
        beacon_entries: Vec<BeaconEntry>,
        #[serde(default, rename = "WinPoStProof", with = "post::json::vec")]
        win_post_proof: Vec<PoStProof>,
        #[serde(rename = "Parents", with = "tipset_keys_json")]
        parents: TipsetKeys,
        #[serde(rename = "ParentWeight")]
        weight: String,
        height: u64,
        #[serde(rename = "ParentStateRoot", with = "cid::json")]
        state_root: Cid,
        #[serde(rename = "ParentMessageReceipts", with = "cid::json")]
        message_receipts: Cid,
        #[serde(with = "cid::json")]
        messages: Cid,
        #[serde(default, rename = "BLSAggregate", with = "signature::json::opt")]
        bls_aggregate: Option<Signature>,
        timestamp: u64,
        #[serde(default, rename = "BlockSig", with = "signature::json::opt")]
        signature: Option<Signature>,
        #[serde(rename = "ForkSignaling")]
        fork_signal: u64,
    }

    let v: BlockHeaderDe = Deserialize::deserialize(deserializer)?;

    Ok(BlockHeader::builder()
        .miner_address(v.miner.parse().map_err(de::Error::custom)?)
        .ticket(v.ticket)
        .beacon_entries(v.beacon_entries)
        .epoch(v.height)
        .win_post_proof(v.win_post_proof)
        .state_root(v.state_root)
        .message_receipts(v.message_receipts)
        .messages(v.messages)
        .timestamp(v.timestamp)
        .fork_signal(v.fork_signal)
        .weight(v.weight.parse().map_err(de::Error::custom)?)
        .parents(v.parents)
        .signature(v.signature)
        .bls_aggregate(v.bls_aggregate)
        .election_proof(v.election_proof)
        .build_and_validate()
        .unwrap())
}