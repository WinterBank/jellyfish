// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::{block_metadata::BlockMetadata, randomness::Randomness};
use aptos_crypto::HashValue;
use move_core_types::account_address::AccountAddress;
use serde::{Deserialize, Serialize};

/// The extended block metadata.
///
/// NOTE for `V0`: this is designed to allow a default block metadata to be represented by this type.
/// By doing so, we can use a single type `BlockMetadataExt` across `StateComputer`,
/// and avoid defining an extra `GenericBlockMetadata` enum for many util functions.
///
/// Implementation also ensures correct conversion to enum `Transaction`:
/// `V0` goes to variant `Transaction::BlockMetadata` and the rest goes to variant `Transaction::BlockMetadataExt`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlockMetadataExt {
    V0(BlockMetadata),
    V1(BlockMetadataWithRandomness),
    V2(BlockMetadataWithMining),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadataWithMining {
    /// The mining difficulty (number of leading zero bits).
    pub difficulty: u32,

    /// The mining reward for this block, represented as an integer (e.g., in smallest currency units).
    pub mining_reward: u64,

    /// The time it took to mine this block (in seconds).
    pub elapsed_time: u64,

    /// The miner's address who mined this block.
    pub miner: AccountAddress,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadataWithRandomness {
    pub id: HashValue,
    pub epoch: u64,
    pub round: u64,
    pub proposer: AccountAddress,
    #[serde(with = "serde_bytes")]
    pub previous_block_votes_bitvec: Vec<u8>,
    pub failed_proposer_indices: Vec<u32>,
    pub timestamp_usecs: u64,
    pub randomness: Option<Randomness>,
}

impl BlockMetadataExt {
    pub fn new_v1(
        id: HashValue,
        epoch: u64,
        round: u64,
        proposer: AccountAddress,
        previous_block_votes_bitvec: Vec<u8>,
        failed_proposer_indices: Vec<u32>,
        timestamp_usecs: u64,
        randomness: Option<Randomness>,
    ) -> Self {
        Self::V1(BlockMetadataWithRandomness {
            id,
            epoch,
            round,
            proposer,
            previous_block_votes_bitvec,
            failed_proposer_indices,
            timestamp_usecs,
            randomness,
        })
    }

    pub fn id(&self) -> HashValue {
        match self {
            BlockMetadataExt::V0(obj) => obj.id(),
            BlockMetadataExt::V1(obj) => obj.id,
            BlockMetadataExt::V2(_) => panic!("V2 does not have id."),
        }
    }

    pub fn timestamp_usecs(&self) -> u64 {
        match self {
            BlockMetadataExt::V0(obj) => obj.timestamp_usecs(),
            BlockMetadataExt::V1(obj) => obj.timestamp_usecs,
            BlockMetadataExt::V2(_) => panic!("V2 does not have timestamp_usecs."),
        }
    }

    pub fn proposer(&self) -> AccountAddress {
        match self {
            BlockMetadataExt::V0(obj) => obj.proposer(),
            BlockMetadataExt::V1(obj) => obj.proposer,
            BlockMetadataExt::V2(obj) => obj.miner,
        }
    }

    pub fn previous_block_votes_bitvec(&self) -> &Vec<u8> {
        match self {
            BlockMetadataExt::V0(obj) => obj.previous_block_votes_bitvec(),
            BlockMetadataExt::V1(obj) => &obj.previous_block_votes_bitvec,
            BlockMetadataExt::V2(_) => panic!("V2 does not have previous_block_votes_bitvec."),
        }
    }

    pub fn failed_proposer_indices(&self) -> &Vec<u32> {
        match self {
            BlockMetadataExt::V0(obj) => obj.failed_proposer_indices(),
            BlockMetadataExt::V1(obj) => &obj.failed_proposer_indices,
            BlockMetadataExt::V2(_) => panic!("V2 does not have failed_proposer_indices."),
        }
    }

    pub fn epoch(&self) -> u64 {
        match self {
            BlockMetadataExt::V0(obj) => obj.epoch(),
            BlockMetadataExt::V1(obj) => obj.epoch,
            BlockMetadataExt::V2(_) => panic!("V2 does not have epoch."),
        }
    }

    pub fn round(&self) -> u64 {
        match self {
            BlockMetadataExt::V0(obj) => obj.round(),
            BlockMetadataExt::V1(obj) => obj.round,
            BlockMetadataExt::V2(_) => panic!("V2 does not have round."),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            BlockMetadataExt::V0(_) => "block_metadata_ext_transaction__v0",
            BlockMetadataExt::V1(_) => "block_metadata_ext_transaction__v1",
            BlockMetadataExt::V2(_) => "block_metadata_ext_transaction__v2",
        }
    }
}

impl From<BlockMetadata> for BlockMetadataExt {
    fn from(v0: BlockMetadata) -> Self {
        BlockMetadataExt::V0(v0)
    }
}

/// Configuration for Proof of Work mining parameters.
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MiningConfig {
    /// The number of leading zero bits required for a valid hash.
    pub difficulty: u32,

    /// The reward formula base for mining rewards.
    pub reward_base: f64,

    /// Target block time in seconds.
    pub block_time_target: u64,

    /// Timestamp of the last mined block (in seconds since UNIX epoch).
    pub last_block_timestamp: u64,
}

impl Default for MiningConfig {
    fn default() -> Self {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        MiningConfig {
            difficulty: 2, // Default difficulty (adjust as needed)
            reward_base: 1.524287, // Reward formula base
            block_time_target: 60, // Target block time (1 block per minute)
            last_block_timestamp: current_time,
        }
    }
}

impl MiningConfig {
    /// Calculates the reward based on the difficulty.
    pub fn calculate_reward(&self) -> f64 {
        self.reward_base.powi(self.difficulty as i32)
    }

    /// Dynamically adjusts the difficulty to maintain the target block time.
    ///
    /// # Arguments
    /// - `actual_time`: The time taken to find the last solution (in seconds).
    pub fn adjust_difficulty(&mut self, actual_time: u64) {
        if actual_time > self.block_time_target {
            // Decrease difficulty if blocks are taking too long
            if self.difficulty > 1 {
                self.difficulty -= 1;
            }
        } else if actual_time < self.block_time_target {
            // Increase difficulty if blocks are being found too quickly
            self.difficulty += 1;
        }
    }

    /// Updates the difficulty based on the current time and the last block timestamp.
    pub fn update_difficulty(&mut self) {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let elapsed_time = current_time - self.last_block_timestamp;
        self.adjust_difficulty(elapsed_time);
        self.last_block_timestamp = current_time;
    }
}
