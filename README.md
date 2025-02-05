
# Concentrated Liqduity Manager Vault Contract

This program is not audited and not intended for mainnet use as is. Please read security issues.

This is a work in progress but builds and passes tests.

# Overview

This is a Solana vault program which supports DEX swapping and [concentrated liquidity](https://cianyyz.mixa.site/CLM/Concentrated%20Liquidity) operations via [Orca](https://www.orca.so).

The vault is divided into shares which can be minted when new funds are deposited into the vault and withdrawn for funds.

This program was built to create a trustless system which could support a [Concentrated Liquidity Manager](https://cianyyz.mixa.site/CLM/CLManager%20Program). Allowing for users to get high yields while being reducing risk to token price volatility.

The vault only trades with a whitelisted set of tokens which prevents trading to newly minted tokens, therefore draining the vault of funds for useless tokens.

The vault will only trade with a whitelisted set of programs for similar reasons.

## Technical

This vault uses [Cross Program Invocation, CPI](https://solana.com/docs/core/cpi), to interact with other programs onchain. It also uses [Program Derived Addresses, PDA](https://solana.com/docs/core/pda) to ensure the vault can make signatures.

Prices are derived from using whirlpool tick indexes instead of relying on oracles. There are price manipulation concerns with the use of [oracles](https://chain.link/education/blockchain-oracles) such as [flash loan attacks.](https://www.aon.com/en/insights/cyber-labs/flash-loan-attacks-a-case-study).

[Unique bumps](https://solana.com/docs/core/pda#canonical-bump) are used to derive PDA addresses to avoid [sea level attacks](https://github.com/coral-xyz/sealevel-attacks/tree/master) such as [bump seed canonicalization exploits](https://github.com/etherfuse/solana-course/blob/main/content/bump-seed-canonicalization.md)

## Known Security Issues

#### Severe

- Ability to swap via a specific whirlpool, this allows vault creator to create a high fee uncompetitive whirlpool and trade solely with it to drain the vault's funds.

- Relies on vault creator to have liquid USDC for withdraws, can simply block withdraws by not trading with USDC. This will be updated when withdraws will trigger decreaseLiquidity and swaps.

- Insecure owner checks, for demonstration purposes only the instruction payer is checked to see if its the owner but this can be manipulated. It would be recommended to implement a [more secure check.](https://github.com/coral-xyz/sealevel-attacks/blob/master/programs/2-owner-checks/secure/src/lib.rs)

## Testing

1. Check Versions

2. Start Test Validator
```
<new terminal>
cd tests
./start-test-validator.sh
```

3. Run Tests
```
anchor test --skip-local-validator
```

## Documentation

```
cargo doc --no-deps --open
```


### Current Limitations:
- Whirlpools must be Token/USD. This vault does not rely on oracles so it assumes all whirlpools have a USD stablecoin and get price directly from whirlpool.

- 1 Position (and therefore whirlpool) at a time. This is to make evaluation easier. However, it means positions must be fully drained before entry into a new position.


### Development Notes

- Make sure to keep Rust on 1.79.0. [Current anchor 0.30.1 and solana are not updated to use the new Cargo.lock version 4 and will have trouble building if the versions are not alligned.](https://github.com/coral-xyz/anchor/issues/3392#issuecomment-2508412018)

- Decided to leave unnecessary orca instructions for easier testing reasons such as verify, init pool, init tick array , etc

- See references, lots of Orca CPI usage has been forked from the repo below with some checks relevant to vault function.


### Upcoming Features

- [Jupiter CPI Swaps](https://station.jup.ag/docs/old/apis/cpi)

- DeFi Loan functionality

- Basic dApp

- Rust Helper Services

### Acknowledgements and References:
* Orca CPI usage, instructions, and testing: https://github.com/orca-so/whirlpool-cpi-sample/tree/main
* Orca Math: https://github.com/orca-so/whirlpools/tree/main
* Initialize and Bump Logic: https://github.com/Clish254/sol-vault/tree/main