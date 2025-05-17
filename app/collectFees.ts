import { ComputeBudgetProgram, Keypair, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import dotenv from "dotenv";
import { _owner, initSdkWithoutOwner } from "./config";
import { Raydium } from "@raydium-io/raydium-sdk-v2";

import { PoolParty } from "../target/types/pool_party";
import { NATIVE_MINT, getAssociatedTokenAddressSync } from "@solana/spl-token";

import { confirmTransaction } from "@solana-developers/helpers";
import { LOOKUP_TABLE, NFT, POOL_NAME } from "./constants";
import { pools, protocolPDAs, raydiumPDAs, tokenVaults } from "./helpers";

dotenv.config();

let manager: Keypair;

let investor: Keypair;

let raydium: Raydium;

const program = anchor.workspace.PoolParty as Program<PoolParty>;

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

export const collectFees = async (
  nft: PublicKey,
  investor: PublicKey,
  poolName: string
) => {
  raydium = await initSdkWithoutOwner({ loadToken: true });
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
    openPositionTokenVault0,
    openPositionTokenVault1,
  } = tokenVaults();

  const {
    tickLowerArrayAddress,
    tickUpperArrayAddress,
    bitmapExtension,
    personalPosition,
    protocolPosition,
  } = await raydiumPDAs(raydium, nft);

  const { poolPositionConfig, poolPosition } = protocolPDAs(program, poolName);
  const poolId = poolMintAWithMintB.toBase58();

  console.log(`poolId`, poolId);
  console.log(`poolMintAWithMintB`, poolMintAWithMintB.toBase58());
  console.log(`tokenVault0`, openPositionTokenVault0.toBase58());
  console.log(`tokenVault1`, openPositionTokenVault1.toBase58());
  console.log(`personalPosition`, personalPosition.toBase58());
  console.log(`NFT`, nft.toBase58());
  console.log(`poolWSolWithMintA`, poolWSolWithMintA.toBase58());
  console.log(`poolWSolWithMintB`, poolWSolWithMintB.toBase58());
  console.log(`tokenVaultInputA`, tokenVaultInputA.toBase58());
  console.log(`tokenVaultInputB`, tokenVaultInputB.toBase58());
  console.log(`poolPositionConfig`, poolPositionConfig.toBase58());
  console.log(`poolPosition`, poolPosition.toBase58());

  const positionNftAccount = getAssociatedTokenAddressSync(
    nft,
    poolPosition,
    true
  );

  console.log(`positionNftAccount`, positionNftAccount.toBase58());

  // get the table from the cluster
  const lookupTableAccount = (
    await provider.connection.getAddressLookupTable(LOOKUP_TABLE, {
      commitment: "confirmed",
    })
  ).value;

  console.log("lookupTableAccount", lookupTableAccount);
  console.log();

  let collectFeesTx = await program.methods
    .collectFees()
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
      collectFeesTx,
    ], // note this is an array of instructions
  }).compileToV0Message([lookupTableAccount]); //lookupTableAccount

  // create a v0 transaction from the v0 message
  const transactionV0 = new anchor.web3.VersionedTransaction(messageV0);
  return transactionV0;
};

const run = async () => {
  manager = _owner;
  provider.connection.getBalance(manager.publicKey).then((balance) => {
    console.log(`After createPoolPosition The lamport balance is ${balance}`);
    console.log();
  });

  investor = manager;

  const transactionV0 = await collectFees(NFT, investor.publicKey, POOL_NAME);

  transactionV0.sign([investor]);

  const tx2 = await provider.connection.sendTransaction(transactionV0);
  await confirmTransaction(provider.connection as any, tx2);
  console.log("================>>>> collectFeesTx");
  console.log(
    `https://explorer.solana.com/tx/${tx2}?cluster=custom&customUrl=http://localhost:8899`
  );
  console.log();
};

run();
