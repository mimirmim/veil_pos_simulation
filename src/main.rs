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

use rand::prelude::*;
use rand_distr::{Distribution, LogNormal};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Write;
use std::ops::Range;

static STAKE_REWARD: usize = 50;
static STAKE_REWARD_10_DENOM: usize = STAKE_REWARD / 10;
static MAX_SUPPLY: usize = 300_000_000;

static SUPER_BLOCK: usize = 43_200;
static REWARD_REDUCTION_BLOCK: usize = 525_960;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Denoms {
    d10: usize,
    d100: usize,
    d1_000: usize,
    d10_000: usize,
}

impl Denoms {
    fn new(balance: &usize, denom_strat: usize) -> Self {
        let mut bal_left = balance.to_owned();
        let mut denoms: Self = Denoms {
            d10: 0,
            d100: 0,
            d1_000: 0,
            d10_000: 0,
        };
        while bal_left >= 10 {
            if bal_left >= 1_000 {
                denoms.d1_000 += 1;
                bal_left -= 1_000;
            } else if bal_left >= 100 {
                denoms.d100 += 1;
                bal_left -= 100;
            } else if bal_left >= 10 {
                denoms.d10 += 1;
                bal_left -= 10;
            }
        }

        denoms.update_denoms(denom_strat);

        denoms
    }

    fn ticket_count(&self) -> f64 {
        let mut count = 0f64;

        // Flat
        count += self.d10 as f64;
        count += self.d100 as f64 * 10.0;
        count += self.d1_000 as f64 * 100.0;
        count += self.d10_000 as f64 * 1000.0;

        // Mine
        // count += self.d10 as f64;
        // count += self.d100 as f64 * 9.5;
        // count += self.d1_000 as f64 * 90.0;
        // count += self.d10_000 as f64 * 850.0;

        // Original
        // count += self.d10 as f64;
        // count += self.d100 as f64 * 9.0;
        // count += self.d1_000 as f64 * 80.0;
        // count += self.d10_000 as f64 * 700.0;

        count
    }

    fn update_denoms(&mut self, denom_strat: usize) {
        // 50/50 1000s, 10,000s
        if denom_strat == 1 {
            if self.d10 >= 10 {
                self.d10 -= 10;
                self.d100 += 1;
            }

            if self.d100 >= 10 {
                self.d100 -= 10;
                self.d1_000 += 1;
            }

            if self.d1_000 > 10 && self.d1_000 > self.d10_000 {
                while self.d1_000 > self.d10_000 && self.d1_000 > 10 {
                    self.d1_000 -= 10;
                    self.d10_000 += 1;
                }
            }
        }

        // Equal across all denoms
        if denom_strat == 2 {
            if self.d10 > 10 && self.d10 > self.d100 {
                while self.d10 > self.d100 && self.d10 > 10 {
                    self.d10 -= 10;
                    self.d100 += 1;
                }
            }

            if self.d100 > 10 && self.d100 > self.d1_000 {
                while self.d100 > self.d1_000 && self.d100 > 10 {
                    self.d100 -= 10;
                    self.d1_000 += 1;
                }
            }

            if self.d1_000 > 10 && self.d1_000 > self.d10_000 {
                while self.d1_000 > self.d10_000 && self.d1_000 > 10 {
                    self.d1_000 -= 10;
                    self.d10_000 += 1;
                }
            }
        }

        // All to 10,000s
        if denom_strat == 3 {
            if self.d10 >= 10 {
                self.d10 -= 10;
                self.d100 += 1;
            }

            if self.d100 >= 10 {
                self.d100 -= 10;
                self.d1_000 += 1;
            }

            if self.d1_000 >= 10 {
                self.d1_000 -= 10;
                self.d10_000 += 1;
            }
        }
    }

    fn get_stake_probability(&self, total_supply: &usize) -> f64 {
        let adjusted_supply = total_supply / 10;
        self.ticket_count() as f64 / adjusted_supply as f64
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct StakeMature {
    reward: usize,
    height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Staker {
    id: usize,
    start_balance: usize,
    start_pct_total: f64,
    balance: usize,
    percent_total: f64,
    change_pct: f64,
    denom_strat: usize,
    total_stake_count: usize,
    conf_stake_count: usize,
    immature_stake_count: usize,
    #[serde(skip_serializing)]
    denoms: Denoms,
    #[serde(skip_serializing)]
    range: Range<f64>,
    #[serde(skip_serializing)]
    stakes_maturing: Vec<StakeMature>,
}

impl Staker {
    fn new(balance: usize, id: usize, start_pct_total: f64, rng: &mut ThreadRng) -> Self {
        let denom_strat = rng.gen_range(1, 4);
        Self {
            id,
            denoms: Denoms::new(&balance, denom_strat.to_owned()),
            denom_strat,
            start_balance: balance.to_owned(),
            start_pct_total,
            balance,
            percent_total: 0.0,
            total_stake_count: 0,
            immature_stake_count: 0,
            range: Range {
                start: 0.0,
                end: 0.0,
            },
            stakes_maturing: Vec::new(),
            change_pct: 0.0,
            conf_stake_count: 0,
        }
    }

    fn hit_stake(&mut self, block_height: usize) {
        let reward = if block_height < REWARD_REDUCTION_BLOCK {
            STAKE_REWARD_10_DENOM
        } else if block_height < REWARD_REDUCTION_BLOCK * 2 {
            STAKE_REWARD_10_DENOM - 1
        } else if block_height < REWARD_REDUCTION_BLOCK * 3 {
            STAKE_REWARD_10_DENOM - 2
        } else if block_height < REWARD_REDUCTION_BLOCK * 4 {
            STAKE_REWARD_10_DENOM - 3
        } else {
            STAKE_REWARD_10_DENOM - 4
        };

        self.stakes_maturing.push(StakeMature {
            reward,
            height: block_height + 1000,
        });
        self.immature_stake_count += 1;
        // self.denoms.d10 += reward;
        self.balance += reward * 10;
        self.total_stake_count += 1;
        self.change_pct =
            (self.balance as f64 - self.start_balance as f64) / self.start_balance as f64 * 100f64;
    }

    fn update(&mut self, total_supply: usize) {
        self.percent_total = self.balance as f64 / total_supply as f64;
    }

    fn are_stakes_maturing(&mut self) -> bool {
        !self.stakes_maturing.is_empty()
    }

    fn stakes_mature(&mut self, block_height: &usize) {
        let mature = self
            .stakes_maturing
            .iter_mut()
            .find(|p| &p.height <= block_height);
        if let Some(mature_stake) = mature {
            self.immature_stake_count -= 1;
            self.conf_stake_count += 1;
            self.denoms.d10 += mature_stake.reward;
            self.stakes_maturing.remove(0);
            self.denoms.update_denoms(self.denom_strat);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Network {
    stakers: Vec<Staker>,
    total_supply: usize,
    block: usize,
}

impl Network {
    fn new() -> Self {
        Self {
            stakers: Vec::new(),
            total_supply: SUPER_BLOCK * STAKE_REWARD * 6, // Start 6 months, gets more stakers.
            block: SUPER_BLOCK * 6,
        }
    }

    fn create_stakers(&mut self, rng: &mut ThreadRng) {
        let mut total_staking_supply = self.total_supply as isize;
        let mut id = 0;
        let log_normal = LogNormal::new(0.1, 1.5).unwrap();
        loop {
            let mut balance = (log_normal.sample(&mut rand::thread_rng()) * 5_000f64) as usize;

            let left_staking_supply = total_staking_supply - balance as isize;
            if left_staking_supply < 0 {
                balance = total_staking_supply as usize;
                total_staking_supply = 0;
            } else {
                total_staking_supply -= balance as isize;
            }

            self.stakers.push(Staker::new(
                balance,
                id,
                balance as f64 / self.total_supply as f64,
                rng,
            ));

            if total_staking_supply == 0 {
                break;
            }

            id += 1;
        }
    }

    fn find_biggest_staker(&self) -> &Staker {
        self.stakers.iter().max_by_key(|p| p.balance).unwrap()
    }

    fn find_smallest_staker(&self) -> &Staker {
        self.stakers.iter().min_by_key(|p| p.balance).unwrap()
    }

    fn find_biggest_change(&self) -> &Staker {
        self.stakers
            .iter()
            .max_by_key(|p| p.change_pct as u64)
            .unwrap()
    }

    fn find_smallest_change(&self) -> &Staker {
        self.stakers
            .iter()
            .min_by_key(|p| p.change_pct as u64)
            .unwrap()
    }

    fn average_change(&self) -> f64 {
        let mut count: f64 = 0.0;
        self.stakers.iter().for_each(|p| count += p.change_pct);
        count / self.stakers.len() as f64
    }

    fn update_stakers(&mut self) {
        let total_supply = self.total_supply;
        self.stakers.iter_mut().for_each(|p| p.update(total_supply));
    }

    fn update_total_supply(&mut self) {
        if self.block >= SUPER_BLOCK + 1000 {
            if self.block < REWARD_REDUCTION_BLOCK {
                self.total_supply += 50;
            } else if self.block < REWARD_REDUCTION_BLOCK * 2 {
                self.total_supply += 40;
            } else if self.block < REWARD_REDUCTION_BLOCK * 3 {
                self.total_supply += 30;
            } else if self.block < REWARD_REDUCTION_BLOCK * 4 {
                self.total_supply += 20;
            } else {
                self.total_supply += 10;
            }
        }
    }
}

fn stake(network: &mut Network, rng: &mut ThreadRng) {
    let mut start = 0.0;
    for mut staker in &mut network.stakers {
        if staker.are_stakes_maturing() {
            staker.stakes_mature(&network.block);
        }

        staker.range = Range {
            start: 0.0,
            end: 0.0,
        };
        let pct = staker.denoms.get_stake_probability(&network.total_supply);
        let range = start..start + pct;
        staker.range = range;
        start += pct;
    }

    let winning_pct = rng.gen_range(0.0, start);
    let winner = network
        .stakers
        .iter_mut()
        .find(|p| p.range.contains(&winning_pct));

    if let Some(winner) = winner {
        winner.hit_stake(network.block);
    } else {
        println!("Impossibruuu!");
    }
}

fn main() {
    println!("Starting...");
    let mut rng: ThreadRng = rand::thread_rng();

    println!("Generating network.");
    let mut network: Network = Network::new();

    println!("Generating stakers.");
    network.create_stakers(&mut rng);

    println!("{} stakers generated.", network.stakers.len());

    let end_block = REWARD_REDUCTION_BLOCK * 10;
    println!("Generating history to block {}.", end_block);
    while network.block <= end_block {
        stake(&mut network, &mut rng);
        network.block += 1;
        network.update_total_supply();

        if network.block % 100 == 0 {
            let pct_done = network.block as f64 / end_block as f64 * 100.0;
            print!("\rAt block {} of {}.", network.block, end_block);

            print!(" [");
            let mut pct_check = 0.0;
            while pct_check != 100.0 {
                if pct_done >= pct_check {
                    print!("#");
                } else {
                    print!("-");
                }
                pct_check += 2.5;
            }
            print!("]");

            print!(" {:.2}%", pct_done);
            io::stdout().flush().unwrap();
        }

        if network.block == end_block {
            print!(
                "\rAt block {} of {}. [########################################] 100.00% done!",
                network.block, end_block
            );
            io::stdout().flush().unwrap();
        }
    }
    network.update_stakers();
    println!("\nBlockchain history generated.");

    let json = serde_json::to_string(&network.stakers).unwrap();
    let file_name = "data.json";
    fs::write("data.json", json).unwrap();
    println!("JSON written to file {} in the base directory.", file_name);
}
