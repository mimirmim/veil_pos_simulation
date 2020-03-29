- Add in configuration options through command line only.
- Split up the binary into a library
- Create a new binary which is meant for RingCT PoS staking with all it's 
differences from Zerocoin PoS.
- When denoms move around, they need to be made immature for 1000 blocks.
- Option if a denom strategy is losing too much, it'll change strategies, but
always up 1. Therefore, strategies should be arranged from 10s -> 10,000s.
- BUG: When a stake fails because it was orphaned, it doesn't find another staker.