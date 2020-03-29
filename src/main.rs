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
// static MAX_SUPPLY: usize = 300_000_000;

static SUPER_BLOCK: usize = 43_200;
static REWARD_REDUCTION_BLOCK: usize = 525_960;

static D10_MOD: f64 = 1.0;
static D100_MOD: f64 = 9.0;
static D1_000_MOD: f64 = 80.0;
static D10_000_MOD: f64 = 700.0;

#[derive(Debug)]
enum Denom {
    D10 = 10,
    D100 = 100,
    D1000 = 1_000,
    D10000 = 10_000,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Denoms {
    d10: usize,
    d100: usize,
    d1_000: usize,
    d10_000: usize,
}

impl Denoms {
    fn new(balance: usize, denom_strat: usize) -> Self {
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

        count += self.d10 as f64 * D10_MOD;
        count += self.d100 as f64 * D100_MOD;
        count += self.d1_000 as f64 * D1_000_MOD;
        count += self.d10_000 as f64 * D10_000_MOD;

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

    fn stake_probability(&self, total_supply: usize) -> f64 {
        let adjusted_supply = total_supply / 10;
        self.ticket_count() as f64 / adjusted_supply as f64
    }

    fn probability(&self, denom: &Denom) -> f64 {
        use Denom::*;
        match denom {
            D10 => self.d10_probability(),
            D100 => self.d100_probability(),
            D1000 => self.d1_000_probability(),
            D10000 => self.d10_000_probability(),
        }
    }

    fn d10_probability(&self) -> f64 {
        (self.d10 as f64 * D10_MOD) / self.ticket_count()
    }

    fn d100_probability(&self) -> f64 {
        (self.d100 as f64 * D100_MOD) / self.ticket_count()
    }

    fn d1_000_probability(&self) -> f64 {
        (self.d1_000 as f64 * D1_000_MOD) / self.ticket_count()
    }

    fn d10_000_probability(&self) -> f64 {
        (self.d10_000 as f64 * D10_000_MOD) / self.ticket_count()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ImmatureBalance {
    is_stake: bool,
    reward: usize,
    height: usize,
    mature_height: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct Staker {
    id: usize,
    start_balance: usize,
    start_pct_total: f64,
    balance_spendable: usize,
    balance_unconfirmed: usize,
    balance_immature: usize,
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
    immature: Vec<ImmatureBalance>,
}

impl Staker {
    fn new(balance: usize, id: usize, start_pct_total: f64, rng: &mut ThreadRng) -> Self {
        let denom_strat = rng.gen_range(1, 4);
        Self {
            id,
            denoms: Denoms::new(balance, denom_strat.to_owned()),
            denom_strat,
            start_balance: balance.to_owned(),
            start_pct_total,
            balance_spendable: balance,
            balance_unconfirmed: 0,
            balance_immature: 0,
            percent_total: 0.0,
            total_stake_count: 0,
            immature_stake_count: 0,
            range: Range {
                start: 0.0,
                end: 0.0,
            },
            immature: Vec::new(),
            change_pct: 0.0,
            conf_stake_count: 0,
        }
    }

    fn hit_stake(&mut self, block_height: usize, mut rng: &mut ThreadRng) {
        let reward = if block_height < REWARD_REDUCTION_BLOCK {
            STAKE_REWARD
        } else if block_height < REWARD_REDUCTION_BLOCK * 2 {
            STAKE_REWARD - 10
        } else if block_height < REWARD_REDUCTION_BLOCK * 3 {
            STAKE_REWARD - 20
        } else if block_height < REWARD_REDUCTION_BLOCK * 4 {
            STAKE_REWARD - 30
        } else {
            STAKE_REWARD - 40
        };

        self.immature.push(ImmatureBalance {
            is_stake: true,
            reward,
            height: block_height,
            mature_height: block_height + 30,
        });
        self.immature_stake_count += 1;
        self.balance_spendable += reward;
        self.total_stake_count += 1;
        self.change_pct = (self.balance_spendable as f64 - self.start_balance as f64)
            / self.start_balance as f64
            * 100f64;

        self.lock_denom(block_height, &mut rng);
    }

    fn lock_denom(&mut self, block_height: usize, rng: &mut ThreadRng) {
        use Denom::*;

        #[derive(Debug)]
        struct DenomRange {
            denom: Denom,
            range: Range<f64>,
        };

        let mut denom_ranges: Vec<DenomRange> = Vec::new();
        if self.denoms.d10 > 0 {
            denom_ranges.push(DenomRange {
                denom: D10,
                range: Range {
                    start: 0.0,
                    end: 0.0,
                },
            })
        }
        if self.denoms.d100 > 0 {
            denom_ranges.push(DenomRange {
                denom: D100,
                range: Range {
                    start: 0.0,
                    end: 0.0,
                },
            })
        }
        if self.denoms.d1_000 > 0 {
            denom_ranges.push(DenomRange {
                denom: D1000,
                range: Range {
                    start: 0.0,
                    end: 0.0,
                },
            })
        }
        if self.denoms.d10_000 > 0 {
            denom_ranges.push(DenomRange {
                denom: D10000,
                range: Range {
                    start: 0.0,
                    end: 0.0,
                },
            })
        }

        let mut start = 0.0;
        for mut denom_range in &mut denom_ranges {
            let pct = self.denoms.probability(&denom_range.denom);
            let range = start..start + pct;
            denom_range.range = range;
            start += pct;
        }

        let winning_pct = rng.gen_range(0.0, start);
        let winner = denom_ranges
            .iter_mut()
            .find(|p| p.range.contains(&winning_pct));

        if let Some(winner) = winner {
            // TODO: Make ImmatureBalance once, update after.
            match winner.denom {
                D10 => {
                    self.denoms.d10 -= 1;
                    self.immature.push(ImmatureBalance {
                        is_stake: false,
                        reward: 10,
                        height: block_height,
                        mature_height: block_height + 1000,
                    });
                }
                D100 => {
                    self.denoms.d100 -= 1;
                    self.immature.push(ImmatureBalance {
                        is_stake: false,
                        reward: 100,
                        height: block_height,
                        mature_height: block_height + 1000,
                    });
                }
                D1000 => {
                    self.denoms.d1_000 -= 1;
                    self.immature.push(ImmatureBalance {
                        is_stake: false,
                        reward: 1_000,
                        height: block_height,
                        mature_height: block_height + 1000,
                    });
                }
                D10000 => {
                    self.denoms.d10_000 -= 1;
                    self.immature.push(ImmatureBalance {
                        is_stake: false,
                        reward: 10_000,
                        height: block_height,
                        mature_height: block_height + 1000,
                    });
                }
            }
            self.immature_stake_count += 1;
        } else {
            println!("Impossibruuu!");
        }
    }

    fn update(&mut self, total_supply: usize) {
        self.percent_total = self.balance_spendable as f64 / total_supply as f64;
    }

    fn are_stakes_maturing(&mut self) -> bool {
        !self.immature.is_empty()
    }

    // TODO: Balances may be different now.
    fn mature_balances(&mut self, block_height: usize) {
        let mature = self
            .immature
            .iter_mut()
            .enumerate()
            .find(|p| p.1.mature_height <= block_height);
        if let Some((pos, mature_stake)) = mature {
            if mature_stake.is_stake {
                self.immature_stake_count -= 1;
                self.conf_stake_count += 1;
            }

            let mut balance_left = mature_stake.reward;
            while balance_left > 0 {
                if balance_left >= 10_000 {
                    self.denoms.d10_000 += 1;
                    balance_left -= 10_000;
                }
                if balance_left >= 1_000 {
                    self.denoms.d1_000 += 1;
                    balance_left -= 1_000;
                }
                if balance_left >= 100 {
                    self.denoms.d100 += 1;
                    balance_left -= 100;
                }
                if balance_left >= 10 {
                    self.denoms.d10 += 1;
                    balance_left -= 10;
                }
            }

            self.immature.remove(pos);
            self.denoms.update_denoms(self.denom_strat);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Network {
    stakers: Vec<Staker>,
    total_supply: usize,
    block_height: usize,
}

impl Network {
    fn new() -> Self {
        Self {
            stakers: Vec::new(),
            total_supply: SUPER_BLOCK * STAKE_REWARD * 6, // Start 6 months, gets more stakers.
            block_height: SUPER_BLOCK * 6,
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

    fn update_stakers(&mut self) {
        let total_supply = self.total_supply;
        self.stakers.iter_mut().for_each(|p| p.update(total_supply));
    }

    fn update_total_supply(&mut self) {
        if self.block_height >= SUPER_BLOCK + 1000 {
            if self.block_height < REWARD_REDUCTION_BLOCK {
                self.total_supply += 50;
            } else if self.block_height < REWARD_REDUCTION_BLOCK * 2 {
                self.total_supply += 40;
            } else if self.block_height < REWARD_REDUCTION_BLOCK * 3 {
                self.total_supply += 30;
            } else if self.block_height < REWARD_REDUCTION_BLOCK * 4 {
                self.total_supply += 20;
            } else {
                self.total_supply += 10;
            }
        }
    }

    fn stake(&mut self, mut rng: &mut ThreadRng) {
        let mut start = 0.0;
        for mut staker in &mut self.stakers {
            if staker.are_stakes_maturing() {
                staker.mature_balances(self.block_height);
            }

            staker.range = Range {
                start: 0.0,
                end: 0.0,
            };
            let pct = staker.denoms.stake_probability(self.total_supply);
            let range = start..start + pct;
            staker.range = range;
            start += pct;
        }

        let winning_pct = rng.gen_range(0.0, start);
        let winner = self
            .stakers
            .iter_mut()
            .find(|p| p.range.contains(&winning_pct));

        if let Some(winner) = winner {
            winner.hit_stake(self.block_height, &mut rng);
        } else {
            println!("Impossibruuu!");
        }
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

    let end_block_height = REWARD_REDUCTION_BLOCK * 1;
    let starting_block_height = network.block_height;
    println!(
        "Generating history from block {} to block {}.",
        starting_block_height, end_block_height
    );
    while network.block_height <= end_block_height {
        network.stake(&mut rng);
        network.block_height += 1;
        network.update_total_supply();

        if network.block_height % 100 == 0 {
            let pct_done = (network.block_height - starting_block_height) as f64
                / end_block_height as f64
                * 100.0;
            print!(
                "\rAt block {} of {}.",
                network.block_height, end_block_height
            );

            print!(" [");
            let mut pct_check = 0.0;
            while pct_check as usize != 100 {
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

        if network.block_height == end_block_height {
            print!(
                "\rAt block {} of {}. [########################################] 100.00% done!",
                network.block_height, end_block_height
            );
            io::stdout().flush().unwrap();
        }
    }
    network.update_stakers();
    println!("\nBlockchain history generated.");

    let json = serde_json::to_string(&network.stakers).unwrap();
    let file_name = "data.json";
    fs::write(file_name, json).unwrap();
    println!("JSON written to file {} in the base directory.", file_name);
}
