# Pool Party

A Solana protocol for liquidity pool management built on Raydium concentrated liquidity pools.

## Overview

Pool Party is a Solana protocol that integrates with Raydium's concentrated liquidity pools, allowing users to create, manage, and interact with liquidity positions. The project leverages Anchor framework and Raydium SDK for seamless DeFi interactions.

## Installation

```bash
# Clone the repository
git clone https://github.com/PoolPartyLabs/SolanaHackthonProgramm.git
cd pool-party

# Install dependencies
npm install
```

## Environment Setup

```bash
# Create a .env file with your configuration
cp .env.example .env

# Set up your Solana wallet for development
# Edit the .env file with your wallet keys
```

## Building the Project

```bash
# Build the Anchor program
make build

# Build with devnet features
make build-devnet

# Clean and rebuild
make clean-build
```

## Deployment

```bash
# Deploy to localnet
make deploy-local

# Deploy to devnet (make sure to set the right config first)
make set-config-devnet
make deploy

# Show program details
make program-show
```

## Testing

```bash
# Run all tests with a local validator
make test

# Run tests without starting a local validator
make test-skip-local-validator

# Run tests skipping builds and local validator
make test-skip-local-validator-build
```

## Local Development

```bash
# Set config to local
make set-config-local

# Start a test validator
make start-test-validator

# View logs
make logs

# Airdrop SOL for testing
make airdrop

# Sync Anchor keys
make sync-keys
```

## Raydium Integration

The project integrates with Raydium concentrated liquidity pools through:

1. **Raydium SDK**: Using `@raydium-io/raydium-sdk-v2` for JavaScript/TypeScript interactions
2. **Raydium CPI**: Using the `raydium-clmm-cpi` crate for Rust program interactions

Key functionalities include:
- Creating liquidity positions
- Depositing into pools
- Collecting fees
- Swapping tokens
- Generating trading fees

## Scripts

The `app` directory contains various scripts to interact with the protocol:

- `setup.ts` - Initialize the environment and create pools
- `createPosition.ts` - Create a new liquidity position
- `openPosition.ts` - Open a position for the protocol
- `deposit.ts` - Deposit assets into a position
- `collectFees.ts` - Collect accumulated fees
- `generateFees.ts` - Generate test fees for demonstration
- `getInvestorInfo.ts` - Retrieve investor position information
- `swap.ts` - Perform token swaps

## Mainnet vs Devnet

The project supports both mainnet and devnet environments:

```bash
# Dump program binaries from devnet for local use
make dump-devnet

# Build with devnet features
make build-devnet

# Change configuration between environments
make set-config-devnet
make set-config-local
```

## License  

This project is licensed under the MIT License (MIT). 

---

## About  

**Pool Party** was developed by the team at **Pool Party Labs**, creators of innovative solutions to simplify and optimize liquidity provision in DeFi. Learn more about our work at [pool-party.xyz](https://pool-party.xyz).  

---

## Contact  

For questions or support, reach out to us at [hi@pool-party.xyz](mailto:hi@pool-party.xyz).
