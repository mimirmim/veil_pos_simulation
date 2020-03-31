use crate::amount;
use crate::denom::{self, Denom};
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
pub struct Wallet {
    initial_state: bool,
    denom_strat: u64,
    denom_threshold: u64,
    total_stake_count: u64,
    conf_stake_count: u64,
    transaction_count: u64,
    mature: Vec<Denom>,
    immature: Vec<Denom>,
}

impl Wallet {
    fn init(balance: u64, denom_strat: u64, denom_threshold: u64) -> Self {
        assert!(amount::money_range(balance));

        let mut wallet: Self = Wallet {
            initial_state: true,
            denom_strat,
            denom_threshold,
            total_stake_count: 0,
            conf_stake_count: 0,
            transaction_count: 0,
            mature: Vec::new(),
            immature: Vec::new(),
        };
        // while bal_left >= 10 {
        // let mut power = denom::MAX_POWER
        // while balance != 0 {
        //     if balance >= 2u64.pow(power) {
        //     } else if balance < 2u64.pow(denom::MIN_POWER) {
        //         else
        //     } {
        //
        //         power -= 1;
        //     }
        // }
        // if bal_left >= 1_000 {
        //     wallet.mature.push(
        //         Denom::builder()
        //             .value(1_000 * amount::COIN)
        //             .mature(true)
        //             .build(),
        //     );
        //     bal_left -= 1_000;
        // } else if bal_left >= 100 {
        //     wallet.mature.push(
        //         Denom::builder()
        //             .value(100 * amount::COIN)
        //             .mature(true)
        //             .build(),
        //     );
        //     bal_left -= 100;
        // } else if bal_left >= 10 {
        //     wallet.mature.push(
        //         Denom::builder()
        //             .value(10 * amount::COIN)
        //             .mature(true)
        //             .build(),
        //     );
        //     bal_left -= 10;
        // }
        // }

        // denoms.update_denoms(0);
        //
        wallet.initial_state = false;

        wallet
    }

    // fn ticket_count(&self) -> f64 {
    //     let mut count = 0f64;
    //
    //     count += self.d10 as f64 * D10_MOD;
    //     count += self.d100 as f64 * D100_MOD;
    //     count += self.d1_000 as f64 * D1_000_MOD;
    //     count += self.d10_000 as f64 * D10_000_MOD;
    //
    //     count
    // }
    //
    // fn update_denoms(&mut self, block_height: u64) {
    //     // 50/50 1000s, 10,000s
    //     if self.denom_strat == 1 {
    //         if self.d10 >= 10 {
    //             self.d10 -= 10;
    //             self.d100 += 1;
    //         }
    //
    //         if self.d100 >= 10 {
    //             self.d100 -= 10;
    //             self.d1_000 += 1;
    //         }
    //
    //         if self.d1_000 > 10 && self.d1_000 > self.d10_000 {
    //             while self.d1_000 > self.d10_000 && self.d1_000 > 10 {
    //                 self.d1_000 -= 10;
    //                 self.d10_000 += 1;
    //             }
    //         }
    //     }
    //
    //     // Equal across all denoms
    //     if self.denom_strat == 2 {
    //         if self.d10 > 10 && self.d10 > self.d100 {
    //             while self.d10 > self.d100 && self.d10 > 10 {
    //                 self.d10 -= 10;
    //                 self.d100 += 1;
    //             }
    //         }
    //
    //         if self.d100 > 10 && self.d100 > self.d1_000 {
    //             while self.d100 > self.d1_000 && self.d100 > 10 {
    //                 self.d100 -= 10;
    //                 self.d1_000 += 1;
    //             }
    //         }
    //
    //         if self.d1_000 > 10 && self.d1_000 > self.d10_000 {
    //             while self.d1_000 > self.d10_000 && self.d1_000 > 10 {
    //                 self.d1_000 -= 10;
    //                 self.d10_000 += 1;
    //             }
    //         }
    //     }
    //
    //     if self.denom_strat == 3 {
    //         self.denoms_to_10_000();
    //     }
    //
    //     // All 100s
    //     if self.denom_strat == 4 && self.d10 > 10 && self.d10 > self.d100 {
    //         while self.d10 > self.d100 && self.d10 > 10 {
    //             self.d10 -= 10;
    //             self.d100 += 1;
    //         }
    //     }
    //
    //     // All 100s and 1,000s 50/50
    //     if self.denom_strat == 5 {
    //         if self.d10 >= 10 {
    //             self.d10 -= 10;
    //             self.d100 += 1;
    //         }
    //
    //         if self.d100 > 10 && self.d100 > self.d1_000 {
    //             while self.d100 > self.d1_000 && self.d100 > 10 {
    //                 self.d100 -= 10;
    //                 self.d1_000 += 1;
    //             }
    //         }
    //     }
    //
    //     // Move all to 10s then work up until you are at an optimal amount of denoms
    //     if self.denom_strat == 6 {
    //         if self.initial_state {
    //             self.denoms_to_d10();
    //         }
    //
    //         if self.count() > self.denom_threshold && self.d10 > 10 {
    //             // delete
    //             while self.d10 > 10 {
    //                 self.d10 -= 10;
    //                 if !self.initial_state {
    //                     self.immature.push(ImmatureBalance {
    //                         is_stake: false,
    //                         reward: 100,
    //                         height: block_height,
    //                         mature_height: block_height + 1000,
    //                     });
    //                 } else {
    //                     self.d100 += 1;
    //                 }
    //
    //                 if self.count() < self.denom_threshold {
    //                     break;
    //                 }
    //             }
    //         }
    //
    //         if self.count() > self.denom_threshold && self.d100 > 10 {
    //             while self.d100 > 10 {
    //                 self.d100 -= 10;
    //                 if !self.initial_state {
    //                     self.immature.push(ImmatureBalance {
    //                         is_stake: false,
    //                         reward: 1_000,
    //                         height: block_height,
    //                         mature_height: block_height + 1000,
    //                     });
    //                     self.immature_stake_count += 1;
    //                 } else {
    //                     self.d1_000 += 1;
    //                 }
    //
    //                 if self.count() < self.denom_threshold {
    //                     break;
    //                 }
    //             }
    //         }
    //
    //         if self.count() > self.denom_threshold && self.d1_000 > 10 {
    //             while self.d1_000 > 10 {
    //                 self.d1_000 -= 10;
    //                 if !self.initial_state {
    //                     self.immature.push(ImmatureBalance {
    //                         is_stake: false,
    //                         reward: 10_000,
    //                         height: block_height,
    //                         mature_height: block_height + 1000,
    //                     });
    //                 } else {
    //                     self.d10_000 += 1;
    //                 }
    //
    //                 if self.count() < self.denom_threshold {
    //                     break;
    //                 }
    //             }
    //         }
    //     }
    // }
    //
    // fn denoms_to_d10(&mut self) {
    //     if self.d10_000 >= 1 {
    //         self.d1_000 += self.d10_000 * 10;
    //         self.d10_000 = 0;
    //     }
    //
    //     if self.d1_000 >= 1 {
    //         self.d100 += self.d1_000 * 10;
    //         self.d1_000 = 0;
    //     }
    //
    //     if self.d100 >= 1 {
    //         self.d10 += self.d100 * 10;
    //         self.d100 = 0;
    //     }
    // }
    //
    // fn denoms_to_10_000(&mut self) {
    //     if self.d10 >= 10 {
    //         self.d10 -= 10;
    //         self.d100 += 1;
    //     }
    //
    //     if self.d100 >= 10 {
    //         self.d100 -= 10;
    //         self.d1_000 += 1;
    //     }
    //
    //     if self.d1_000 >= 10 {
    //         self.d1_000 -= 10;
    //         self.d10_000 += 1;
    //     }
    // }
    //
    // fn stake_probability(&self, total_supply: u64) -> f64 {
    //     let adjusted_supply = total_supply / 10;
    //     self.ticket_count() as f64 / adjusted_supply as f64
    // }
    //
    // fn probability(&self, denom: &Denom) -> f64 {
    //     use Denom::*;
    //     match denom {
    //         D10 => self.d10_probability(),
    //         D100 => self.d100_probability(),
    //         D1000 => self.d1_000_probability(),
    //         D10000 => self.d10_000_probability(),
    //     }
    // }
    //
    // fn d10_probability(&self) -> f64 {
    //     (self.d10 as f64 * D10_MOD) / self.ticket_count()
    // }
    //
    // fn d100_probability(&self) -> f64 {
    //     (self.d100 as f64 * D100_MOD) / self.ticket_count()
    // }
    //
    // fn d1_000_probability(&self) -> f64 {
    //     (self.d1_000 as f64 * D1_000_MOD) / self.ticket_count()
    // }
    //
    // fn d10_000_probability(&self) -> f64 {
    //     (self.d10_000 as f64 * D10_000_MOD) / self.ticket_count()
    // }
    //
    // fn count(&self) -> u64 {
    //     let mut count = 0;
    //     count += self.d10;
    //     count += self.d100;
    //     count += self.d1_000;
    //     count += self.d10_000;
    //     count
    // }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_wallet() {
        println!("starting");
        let balance = 1_234_567 * amount::COIN;
        let wallet = Wallet::init(balance, 0, 0);
        println!("mature: {:#?}", wallet.mature.len());
    }

    #[test]
    fn testyt() {
        println!("{}", 1u64 << 52);
    }
}
