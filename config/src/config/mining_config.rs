/// Configuration for Proof of Work mining parameters.
use serde::{Deserialize, Serialize};
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
