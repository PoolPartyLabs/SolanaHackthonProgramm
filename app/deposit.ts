import {
  ComputeBudgetProgram,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import dotenv from "dotenv";
import { _owner, initSdkWithoutOwner } from "./config";
import {
  ApiV3PoolInfoConcentratedItem,
  PoolUtils,
  Raydium,
} from "@raydium-io/raydium-sdk-v2";

import { PoolParty } from "../target/types/pool_party";
import { NATIVE_MINT, getAssociatedTokenAddressSync } from "@solana/spl-token";

import { confirmTransaction } from "@solana-developers/helpers";
import { airdrop } from "./utils";
import { AMM_CONFIG, LOOKUP_TABLE, NFT, POOL_NAME } from "./constants";
import { pools, tokenVaults, raydiumPDAs, protocolPDAs } from "./helpers";

dotenv.config();

let feePayer: Keypair;

let investor: Keypair;

let raydium: Raydium;

const program = anchor.workspace.PoolParty as Program<PoolParty>;

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

investor = Keypair.generate();

export const deposit = async (
  investor: PublicKey,
  nft: PublicKey,
  poolName: string = POOL_NAME
) => {
  raydium = await initSdkWithoutOwner({ loadToken: true });
  const mintInput = NATIVE_MINT;

  const {
    mintA,
    mintB,
    poolMintAWithMintB,
    poolWSolWithMintA,
    poolWSolWithMintB,
  } = pools();

  const {
    tokenVaultInputA,
    tokenVaultInputB,
    depositTokenVault0,
    depositTokenVault1,
    openPositionTokenVault0,
    openPositionTokenVault1,
  } = tokenVaults();

  const {
    tickLowerArrayAddress,
    tickUpperArrayAddress,
    bitmapExtension,
    bitmapExtensionA,
    bitmapExtensionB,
    personalPosition,
    protocolPosition,
  } = await raydiumPDAs(raydium, nft);

  const { poolPositionConfig, poolPosition } = protocolPDAs(program, poolName);
  const poolId = poolMintAWithMintB.toBase58();

  console.log(`poolId`, poolId);
  console.log(`poolMintAWithMintB`, poolMintAWithMintB.toBase58());
  console.log(`tokenVault0`, depositTokenVault0.toBase58());
  console.log(`tokenVault1`, depositTokenVault1.toBase58());
  console.log(`personalPosition`, personalPosition.toBase58());
  console.log(`NFT`, nft.toBase58());
  console.log(`poolWSolWithMintA`, poolWSolWithMintA.toBase58());
  console.log(`poolWSolWithMintB`, poolWSolWithMintB.toBase58());
  console.log(`tokenVaultInputA`, tokenVaultInputA.toBase58());
  console.log(`tokenVaultInputB`, tokenVaultInputB.toBase58());
  console.log(`poolPositionConfig`, poolPositionConfig.toBase58());
  console.log(`poolPosition`, poolPosition.toBase58());

  const dataA = await raydium.clmm.getPoolInfoFromRpc(
    poolWSolWithMintA.toBase58()
  );
  const poolInfoA: ApiV3PoolInfoConcentratedItem = dataA.poolInfo;
  const observationIdA = dataA.computePoolInfo.observationId;

  const dataB = await raydium.clmm.getPoolInfoFromRpc(
    poolWSolWithMintB.toBase58()
  );
  const poolInfoB: ApiV3PoolInfoConcentratedItem = dataB.poolInfo;
  const observationIdB = dataB.computePoolInfo.observationId;

  const tokenAccount1 = getAssociatedTokenAddressSync(mintB, investor);

  console.log(`tokenAccount1`, tokenAccount1.toBase58());

  const poolIdA = poolWSolWithMintA.toBase58();
  const poolIdB = poolWSolWithMintB.toBase58();
  const inputAmount = new anchor.BN(0.001 * LAMPORTS_PER_SOL);
  const baseIn = true;

  const { minAmountOut, remainingAccounts: remainingAccountsA } =
    await PoolUtils.computeAmountOutFormat({
      poolInfo: dataA.computePoolInfo,
      tickArrayCache: dataA.tickData[poolIdA],
      amountIn: inputAmount,
      tokenOut: poolInfoA[baseIn ? "mintB" : "mintA"],
      slippage: 0,
      epochInfo: await raydium.fetchEpochInfo(),
    });

  console.log("remainingAccountsA:", remainingAccountsA);

  const { remainingAccounts: remainingAccountsB } =
    await PoolUtils.computeAmountOutFormat({
      poolInfo: dataB.computePoolInfo,
      tickArrayCache: dataB.tickData[poolIdB],
      amountIn: inputAmount,
      tokenOut: poolInfoB[baseIn ? "mintB" : "mintA"],
      slippage: 0,
      epochInfo: await raydium.fetchEpochInfo(),
    });

  console.log("remainingAccountsB:", remainingAccountsB);
  console.log("minAmountOut:", minAmountOut.amount.toSignificant(6));

  const positionNftAccount = getAssociatedTokenAddressSync(
    NFT,
    poolPosition,
    true
  );

  console.log(`positionNftAccount`, positionNftAccount.toBase58());

  let createInvestorPositionTx = await program.methods
    .createInvestorPosition()
    .accounts({
      investor,
      poolPositionConfig,
      poolVaultToken0Mint: mintA,
      poolVaultToken1Mint: mintB,
    })
    .instruction();

  let depositTx = await program.methods
    .deposit(inputAmount)
    .accounts({
      investor: investor,
      poolPositionConfig,
      poolVaultDepositStableMint: mintInput,
    })
    .instruction();

  // get the table from the cluster
  const lookupTableAccount = (
    await provider.connection.getAddressLookupTable(LOOKUP_TABLE, {
      commitment: "confirmed",
    })
  ).value;

  console.log("lookupTableAccount", lookupTableAccount);
  console.log();

  let swapToRatioDepositTx = await program.methods
    .swapToRatioDeposit(new anchor.BN(0), new anchor.BN(0), baseIn)
    .accounts({
      investor,
      poolPositionConfig,
      ammConfig0: AMM_CONFIG,
      ammConfig1: AMM_CONFIG,
      poolState: poolMintAWithMintB,
      personalPosition,
      poolState0: poolWSolWithMintA,
      poolState1: poolWSolWithMintB,
      poolVault0Input: tokenVaultInputA,
      poolVault1Input: tokenVaultInputB,
      poolVaultToken0Account: depositTokenVault0,
      poolVaultToken1Account: depositTokenVault1,
      observationState0: observationIdA,
      observationState1: observationIdB,
      poolVaultDepositStableMint: mintInput,
      poolVaultToken0Mint: mintA,
      poolVaultToken1Mint: mintB,
    })
    .remainingAccounts([
      { pubkey: bitmapExtensionA, isSigner: false, isWritable: true },
      ...remainingAccountsA.map((i) => ({
        pubkey: i,
        isSigner: false,
        isWritable: true,
      })),

      { pubkey: PublicKey.default, isSigner: false, isWritable: false },

      { pubkey: bitmapExtensionB, isSigner: false, isWritable: true },
      ...remainingAccountsB.map((i) => ({
        pubkey: i,
        isSigner: false,
        isWritable: true,
      })),
    ])
    .instruction();

  let increaseLiquidityTx = await program.methods
    .increaseLiquidity()
    .accounts({
      investor,
      poolPositionConfig,
      positionNftAccount,
      poolState: poolMintAWithMintB,
      personalPosition,
      protocolPosition,
      tickArrayLower: tickLowerArrayAddress.toBase58(),
      tickArrayUpper: tickUpperArrayAddress.toBase58(),
      tokenVault0: openPositionTokenVault0,
      tokenVault1: openPositionTokenVault1,
      poolVaultTokenAMint: mintA,
      poolVaultTokenBMint: mintB,
    })
    .remainingAccounts([
      { pubkey: bitmapExtension, isSigner: false, isWritable: true },
    ])
    .instruction();

  const latestBlockhash = await provider.connection.getLatestBlockhash();

  const messageV0 = new anchor.web3.TransactionMessage({
    payerKey: investor,
    recentBlockhash: latestBlockhash.blockhash,
    instructions: [
      ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 100 }),
      ComputeBudgetProgram.setComputeUnitLimit({ units: 700_000 }),
      createInvestorPositionTx,
    ], // note this is an array of instructions
  }).compileToV0Message([lookupTableAccount]);

  // create a v0 transaction from the v0 message
  const transaction0 = new anchor.web3.VersionedTransaction(messageV0);

  const latestBlockhash1 = await provider.connection.getLatestBlockhash();
  const messageV01 = new anchor.web3.TransactionMessage({
    payerKey: investor,
    recentBlockhash: latestBlockhash1.blockhash,
    instructions: [
      ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 100 }),
      ComputeBudgetProgram.setComputeUnitLimit({ units: 700_000 }),
      depositTx,
      swapToRatioDepositTx,
      increaseLiquidityTx,
    ], // note this is an array of instructions
  }).compileToV0Message([lookupTableAccount]);

  // create a v0 transaction from the v0 message
  const transaction1 = new anchor.web3.VersionedTransaction(messageV01);

  return { transaction0, transaction1 };
};

const run = async () => {
  feePayer = _owner;
  const investorPublicKey = investor.publicKey;
  for (let i = 0; i < 10; i++) {
    await airdrop(provider.connection, investorPublicKey);
  }
  provider.connection.getBalance(investorPublicKey).then((balance) => {
    console.log(
      `Initial investor The lamport balance is ${balance} ${investorPublicKey.toBase58()}`
    );
    console.log();
  });

  const { transaction0, transaction1 } = await deposit(
    investorPublicKey,
    NFT,
    POOL_NAME
  );

  transaction0.sign([investor]);
  transaction1.sign([investor]);

  const tx2 = await provider.connection.sendTransaction(transaction0);
  await confirmTransaction(provider.connection as any, tx2);
  console.log("createInvestorPositionTx");
  console.log(
    `https://explorer.solana.com/tx/${tx2}?cluster=custom&customUrl=http://localhost:8899`
  );
  console.log();

  const tx3 = await provider.connection.sendTransaction(transaction1);
  await confirmTransaction(provider.connection as any, tx3);
  console.log(
    `https://explorer.solana.com/tx/${tx3}?cluster=custom&customUrl=http://localhost:8899`
  );

  console.log();
};

run();
