- Add in configuration options through command line only.
- Split up the binary into a library
- Create a new binary which is meant for RingCT PoS staking with all it's 
differences from Zerocoin PoS.
- BUG: When a stake fails because it was orphaned, it doesn't find another staker.
- Code desperately needs to be cleaned up at this point. Its just a mess.
- Ability to choose either log distribution, normal, or flat RNG.