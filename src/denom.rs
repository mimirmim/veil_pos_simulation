// Copyright 2020 Mimir (mimirmim)
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice,
// this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
// this list of conditions and the following disclaimer in the documentation
// and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.

use crate::amount;
use serde::{Deserialize, Serialize};

pub static DENOM_BRACKET_MOD: f64 = 0.0;
pub static DENOM_MIN: u64 = 1u64 << 32;
pub static DENOM_MAX: u64 = 1u64 << 52;
pub static DENOM_SHIFT: u32 = 2;

pub static DENOM_THRESHOLD_MIN: u64 = 0;
pub static DENOM_THRESHOLD_MAX: u64 = 20_000;

#[derive(Debug)]
enum DenomValue {
    D10 = 10,
    D100 = 100,
    D1000 = 1_000,
    D10000 = 10_000,
}

enum DenomStrategy {
    Only10,
    Only100,
    Only1000,
    Only10000,
    Half10And100,
    Half100And1000,
    Half1000And10000,
    AllEqual,
    Optimal,
}

pub struct DenomBuilder {
    value: u64,
    stake_mod: f64,
    is_stake: bool,
    is_mature: bool,
    created_height: u64,
    mature_height: u64,
}

impl DenomBuilder {
    pub fn new() -> Self {
        Self {
            value: 0,
            stake_mod: 1.0,
            is_stake: false,
            is_mature: false,
            created_height: 0,
            mature_height: 0,
        }
    }

    pub fn value(mut self, v: u64) -> Self {
        assert!(amount::money_range(v));
        self.value = v;
        self
    }

    pub fn base_value(mut self, v: f64) -> Self {
        self.value((v * amount::COIN as f64) as u64)
    }

    pub fn stake_mod(mut self, v: f64) -> Self {
        self.stake_mod = v;
        self
    }

    pub fn stake(mut self, v: bool) -> Self {
        self.is_stake = v;
        self
    }

    pub fn mature(mut self, v: bool) -> Self {
        self.is_mature = v;
        self
    }

    pub fn created_height(mut self, v: u64) -> Self {
        self.created_height = v;
        self
    }

    pub fn mature_height(mut self, v: u64) -> Self {
        self.mature_height = v;
        self
    }

    pub fn build(self) -> Denom {
        Denom {
            value: self.value,
            stake_mod: self.stake_mod,
            is_stake: self.is_stake,
            is_mature: self.is_mature,
            created_height: self.created_height,
            mature_height: self.mature_height,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Denom {
    value: u64,
    stake_mod: f64,
    is_stake: bool,
    is_mature: bool,
    created_height: u64,
    mature_height: u64,
}

impl Denom {
    pub fn new(
        sat_value: u64,
        stake_mod: f64,
        is_stake: bool,
        is_mature: bool,
        created_height: u64,
        mature_height: u64,
    ) -> Self {
        assert!(amount::money_range(sat_value));
        Self {
            value: sat_value,
            stake_mod,
            is_stake,
            is_mature,
            created_height,
            mature_height,
        }
    }

    pub fn builder() -> DenomBuilder {
        DenomBuilder::new()
    }

    pub fn set_mature(&mut self, is_mature: bool) {
        self.is_mature = is_mature;
    }

    pub fn base_value(&self) -> f64 {
        self.value as f64 / amount::COIN as f64
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn stake_mod(&self) -> f64 {
        self.stake_mod
    }

    pub fn is_stake(&self) -> bool {
        self.is_stake
    }

    pub fn is_mature(&self) -> bool {
        self.is_mature
    }

    pub fn created_height(&self) -> u64 {
        self.created_height
    }

    pub fn mature_height(&self) -> u64 {
        self.mature_height
    }

    pub fn weight(&self) -> u64 {
        let mut bracket = DENOM_MIN;
        let mut modifier = 0.0;
        let mut weight = 0;
        if self.value > DENOM_MIN {
            while bracket <= DENOM_MAX {
                if self.value > bracket && self.value < bracket << 2 {
                    let weight_modifier = 1.0 - (modifier * DENOM_BRACKET_MOD);
                    weight = ((bracket + 1) as f64 * weight_modifier) as u64;

                    break;
                }
                bracket <<= 2;
                modifier += 1.0;
            }
        }
        weight
    }

    pub fn can_stake(&self) -> bool {
        self.value() >= DENOM_MIN
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amount;

    #[test]
    fn test_denom_no_weight() {
        let denom = Denom::builder().value(DENOM_MIN).build();
        assert_eq!(denom.weight(), 0);
    }

    #[test]
    fn test_denom_min_weight() {
        let denom = Denom::builder().value(DENOM_MIN + 1).build();
        assert_eq!(denom.weight(), 4_294_967_297);
    }

    #[test]
    fn test_denom_next_weight() {
        let shift: u64 = 1u64 << 34;
        let denom = Denom::builder().value(shift + 1).build();
        assert_eq!(denom.weight(), 17_179_869_185);
    }

    #[test]
    #[should_panic]
    fn test_denom_builder_money_range() {
        Denom::builder().value(amount::MAX_MONEY + 1).build();
    }

    #[test]
    fn printy() {
        let mut max = 2u64 << 34;
        println!("max: {}", max);
        max <<= 2;
        println!("max: {}", max);
    }
}
