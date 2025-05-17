import { PublicKey } from "@solana/web3.js";
import { Program } from "@coral-xyz/anchor";
import { _owner } from "./config";
import {
  ApiV3PoolInfoConcentratedItem,
  Raydium,
  TickUtils,
} from "@raydium-io/raydium-sdk-v2";

import { PoolParty } from "../target/types/pool_party";
import { NATIVE_MINT } from "@solana/spl-token";

import { getTickArrayBitmapAddress, i32ToBytes } from "./utils";
import {
  AMM_CONFIG,
  CLMM_PROGRAM_ID,
  MINT_A,
  MINT_B,
  NFT,
  POOL_NAME,
  TICK_LOWER,
  TICK_UPPER,
} from "./constants";

export const pools = () => {
  let mintA = MINT_A;
  let mintB = MINT_B;
  if (mintA.toBase58() > mintB.toBase58()) {
    const temp = mintA;
    mintA = mintB;
    mintB = temp;
  }
  const [poolMintAWithMintB] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool"),
      AMM_CONFIG.toBuffer(),
      mintA.toBuffer(),
      mintB.toBuffer(),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [poolWSolWithMintA] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool"),
      AMM_CONFIG.toBuffer(),
      NATIVE_MINT.toBuffer(),
      mintA.toBuffer(),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [poolWSolWithMintB] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool"),
      AMM_CONFIG.toBuffer(),
      NATIVE_MINT.toBuffer(),
      mintB.toBuffer(),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  return {
    mintA,
    mintB,
    poolMintAWithMintB,
    poolWSolWithMintA,
    poolWSolWithMintB,
  };
};

export const tokenVaults = () => {
  const {
    mintA,
    mintB,
    poolMintAWithMintB,
    poolWSolWithMintA,
    poolWSolWithMintB,
  } = pools();

  const [tokenVaultInputA] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool_vault"),
      poolWSolWithMintA.toBuffer(),
      NATIVE_MINT.toBuffer(),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [tokenVaultInputB] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool_vault"),
      poolWSolWithMintB.toBuffer(),
      NATIVE_MINT.toBuffer(),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [depositTokenVault0] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool_vault"), poolWSolWithMintA.toBuffer(), mintA.toBuffer()],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [depositTokenVault1] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool_vault"), poolWSolWithMintB.toBuffer(), mintB.toBuffer()],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [openPositionTokenVault0] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool_vault"),
      poolMintAWithMintB.toBuffer(),
      mintA.toBuffer(),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [openPositionTokenVault1] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("pool_vault"),
      poolMintAWithMintB.toBuffer(),
      mintB.toBuffer(),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );
  return {
    tokenVaultInputA,
    tokenVaultInputB,
    depositTokenVault0,
    depositTokenVault1,
    openPositionTokenVault0,
    openPositionTokenVault1,
  };
};

export const raydiumPDAs = async (raydium: Raydium, nft: PublicKey) => {
  const { poolWSolWithMintA, poolWSolWithMintB, poolMintAWithMintB } = pools();
  let poolInfo: ApiV3PoolInfoConcentratedItem;
  const data = await raydium.clmm.getPoolInfoFromRpc(
    poolMintAWithMintB.toBase58()
  );
  poolInfo = data.poolInfo;

  const [tickLowerArrayAddress, tickUpperArrayAddress] = [
    TickUtils.getTickArrayAddressByTick(
      new PublicKey(poolInfo.programId),
      new PublicKey(poolInfo.id),
      TICK_LOWER,
      poolInfo.config.tickSpacing
    ),
    TickUtils.getTickArrayAddressByTick(
      new PublicKey(poolInfo.programId),
      new PublicKey(poolInfo.id),
      TICK_UPPER,
      poolInfo.config.tickSpacing
    ),
  ];

  const tickArrayLowerStartIndex = TickUtils.getTickArrayStartIndexByTick(
    TICK_LOWER,
    poolInfo.config.tickSpacing
  );

  const tickArrayUpperStartIndex = TickUtils.getTickArrayStartIndexByTick(
    TICK_UPPER,
    poolInfo.config.tickSpacing
  );

  const [bitmapExtension, _bump111] = await getTickArrayBitmapAddress(
    poolMintAWithMintB,
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [bitmapExtensionA, _bump111A] = await getTickArrayBitmapAddress(
    poolWSolWithMintA,
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [bitmapExtensionB, _bump111B] = await getTickArrayBitmapAddress(
    poolWSolWithMintB,
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [personalPosition] = PublicKey.findProgramAddressSync(
    [Buffer.from("position"), nft.toBuffer()],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  const [protocolPosition] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("position"),
      poolMintAWithMintB.toBuffer(),
      i32ToBytes(TICK_LOWER),
      i32ToBytes(TICK_UPPER),
    ],
    new PublicKey(CLMM_PROGRAM_ID)
  );

  return {
    tickLowerArrayAddress,
    tickUpperArrayAddress,
    tickArrayLowerStartIndex,
    tickArrayUpperStartIndex,
    bitmapExtension,
    bitmapExtensionA,
    bitmapExtensionB,
    personalPosition,
    protocolPosition,
  };
};

export const observationIds = async (raydium: Raydium) => {
  const { poolWSolWithMintA, poolWSolWithMintB } = pools();
  const dataA = await raydium.clmm.getPoolInfoFromRpc(
    poolWSolWithMintA.toBase58()
  );
  const observationIdA = dataA.computePoolInfo.observationId;

  const dataB = await raydium.clmm.getPoolInfoFromRpc(
    poolWSolWithMintB.toBase58()
  );
  const observationIdB = dataB.computePoolInfo.observationId;

  return {
    observationIdA,
    observationIdB,
  };
};

export const protocolPDAs = (program: Program<PoolParty>, poolName: string) => {
  const [poolPositionConfig] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool_position_config:"), Buffer.from(poolName)],
    program.programId
  );

  const [poolPosition] = PublicKey.findProgramAddressSync(
    [Buffer.from("pool_position:"), poolPositionConfig.toBuffer()],
    program.programId
  );

  return {
    poolPositionConfig,
    poolPosition,
  };
};
