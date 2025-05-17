import {
  ComputeBudgetProgram,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
} from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { Program, web3 } from "@coral-xyz/anchor";
import * as splToken from "@solana/spl-token";
import dotenv from "dotenv";
import { _owner, initSdk } from "./config";
import {
  ApiV3PoolInfoConcentratedItem,
  Raydium,
  TickUtils,
} from "@raydium-io/raydium-sdk-v2";
import { confirmTransaction } from "@solana-developers/helpers";

import { PoolParty } from "../target/types/pool_party";
import { NATIVE_MINT, getAssociatedTokenAddressSync } from "@solana/spl-token";
import {
  getNftMetadataAddress,
  getTickArrayBitmapAddress,
  i32ToBytes,
} from "./utils";
import {
  AMM_CONFIG,
  CLMM_PROGRAM_ID,
  MINT_A,
  MINT_B,
  POOL_NAME,
  TICK_LOWER,
  TICK_UPPER,
} from "./constants";
import {
  observationIds,
  pools,
  protocolPDAs,
  raydiumPDAs,
  tokenVaults,
} from "./helpers";

dotenv.config();

let feePayer: Keypair;

let manager: Keypair;

let raydium: Raydium;

const program = anchor.workspace.PoolParty as Program<PoolParty>;

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const run = async () => {
  manager = _owner;
  provider.connection.getBalance(manager.publicKey).then((balance) => {
    console.log(`After openPosition The lamport balance is ${balance}`);
    console.log();
  });
  const NFT = Keypair.generate();
  raydium = await initSdk({ owner: feePayer, loadToken: true });

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
    tickArrayLowerStartIndex,
    tickArrayUpperStartIndex,
    bitmapExtension,
    personalPosition,
    protocolPosition,
  } = await raydiumPDAs(raydium, NFT.publicKey);
  const { poolPositionConfig, poolPosition } = protocolPDAs(program, POOL_NAME);
  const poolId = poolMintAWithMintB.toBase58();

  console.log(`poolId`, poolId);
  console.log(`poolMintAWithMintB`, poolMintAWithMintB.toBase58());
  console.log(`openPositionTokenVault0`, openPositionTokenVault0.toBase58());
  console.log(`openPositionTokenVault1`, openPositionTokenVault1.toBase58());
  console.log(`depositTokenVault0`, depositTokenVault0.toBase58());
  console.log(`depositTokenVault1`, depositTokenVault1.toBase58());
  console.log(`personalPosition`, personalPosition.toBase58());
  console.log(`NFT.publicKey`, NFT.publicKey.toBase58());
  console.log(`poolWSolWithMintA`, poolWSolWithMintA.toBase58());
  console.log(`poolWSolWithMintB`, poolWSolWithMintB.toBase58());
  console.log(`tokenVaultInputA`, tokenVaultInputA.toBase58());
  console.log(`tokenVaultInputB`, tokenVaultInputB.toBase58());
  console.log(`poolPositionConfig`, poolPositionConfig.toBase58());
  console.log(`poolPosition`, poolPosition.toBase58());

  const positionNftAccount = getAssociatedTokenAddressSync(
    NFT.publicKey,
    poolPosition,
    true
  );
  const metadataAccount = (await getNftMetadataAddress(NFT.publicKey))[0];

  const tokenAccount0 = getAssociatedTokenAddressSync(mintA, manager.publicKey);

  console.log(`tokenAccount0`, tokenAccount0.toBase58());

  const tokenAccount1 = getAssociatedTokenAddressSync(mintB, manager.publicKey);

  console.log(`tokenAccount1`, tokenAccount1.toBase58());

  let createPositionTx = await program.methods
    .createPosition(
      POOL_NAME,
      TICK_LOWER,
      TICK_UPPER,
      poolMintAWithMintB,
      depositTokenVault0,
      depositTokenVault1,
      mintA,
      mintB
    )
    .accounts({
      manager: manager.publicKey,
    })
    .signers([manager])
    .rpc({ commitment: "confirmed" });

  const tx1 = await confirmTransaction(
    provider.connection as any,
    createPositionTx,
    "confirmed"
  );

  console.log();
  console.log(
    "createPositionTx",
    `https://explorer.solana.com/tx/${tx1}?cluster=custom&customUrl=http://localhost:8899`
  );
  console.log();

  let createPositionVaultsTx = await program.methods
    .createPositionVaults()
    .accounts({
      manager: manager.publicKey,
      poolPosition: poolPosition,
      vault0Mint: mintA,
      vault1Mint: mintB,
    })
    .signers([manager])
    .rpc({ commitment: "confirmed" });

  const tx11 = await confirmTransaction(
    provider.connection as any,
    createPositionVaultsTx,
    "confirmed"
  );

  console.log();
  console.log(
    "createPositionVaultsTx",
    `https://explorer.solana.com/tx/${tx11}?cluster=custom&customUrl=http://localhost:8899`
  );
  console.log();

  let openPositionTx = await program.methods
    .openPosition(
      new anchor.BN(600 * LAMPORTS_PER_SOL),
      new anchor.BN(1832 * LAMPORTS_PER_SOL),
      TICK_LOWER,
      TICK_UPPER,
      tickArrayLowerStartIndex,
      tickArrayUpperStartIndex
    )
    .accounts({
      manager: manager.publicKey,
      poolPositionConfig,
      positionNftMint: NFT.publicKey,
      positionNftAccount,
      metadataAccount,
      poolState: poolMintAWithMintB,
      protocolPosition,
      tickArrayLower: tickLowerArrayAddress.toBase58(),
      tickArrayUpper: tickUpperArrayAddress.toBase58(),
      personalPosition,
      tokenAccount0,
      tokenAccount1,
      tokenVault0: openPositionTokenVault0,
      tokenVault1: openPositionTokenVault1,
      vault0Mint: mintA,
      vault1Mint: mintB,
    })
    .remainingAccounts([
      { pubkey: bitmapExtension, isSigner: false, isWritable: true },
    ])
    .preInstructions([
      ComputeBudgetProgram.setComputeUnitLimit({ units: 500_000 }),
    ])
    .signers([manager, NFT])
    .rpc({ commitment: "confirmed" });

  const tx = await confirmTransaction(
    provider.connection as any,
    openPositionTx,
    "confirmed"
  );
  console.log(
    "openPositionTx",
    `https://explorer.solana.com/tx/${tx}?cluster=custom&customUrl=http://localhost:8899`
  );

  const { observationIdA, observationIdB } = await observationIds(raydium);

  await createLookUpTable([
    // inputAssTokenAccount,
    poolPositionConfig,
    manager.publicKey,
    positionNftAccount,
    poolMintAWithMintB,
    personalPosition,
    protocolPosition,
    tickLowerArrayAddress,
    tickUpperArrayAddress,
    AMM_CONFIG,
    poolWSolWithMintA,
    poolWSolWithMintB,
    tokenVaultInputA,
    tokenVaultInputB,
    openPositionTokenVault0,
    openPositionTokenVault1,
    depositTokenVault0,
    depositTokenVault1,
    mintInput,
    mintA,
    mintB,
    observationIdA,
    observationIdB,
  ]);
};

const createLookUpTable = async (addresses: PublicKey[]) => {
  const slot = await provider.connection.getSlot();
  const [lookupTableInst, lookupTableAddress] =
    anchor.web3.AddressLookupTableProgram.createLookupTable({
      authority: manager.publicKey,
      payer: manager.publicKey,
      recentSlot: slot - 1,
    });

  console.log(
    "\n Created lookup table with address:",
    lookupTableAddress.toBase58()
  );
  console.log();

  let blockhash = await provider.connection
    .getLatestBlockhash()
    .then((res) => res.blockhash);

  const extendInstruction =
    anchor.web3.AddressLookupTableProgram.extendLookupTable({
      payer: manager.publicKey,
      authority: manager.publicKey,
      lookupTable: lookupTableAddress,
      addresses,
    });

  const messageV0 = new web3.TransactionMessage({
    payerKey: manager.publicKey,
    recentBlockhash: blockhash,
    instructions: [lookupTableInst, extendInstruction],
  }).compileToV0Message();

  const transaction = new web3.VersionedTransaction(messageV0);

  //// sign your transaction with the required `Signers`
  transaction.sign([manager]);

  const txId = await provider.connection.sendTransaction(transaction, {
    preflightCommitment: "confirmed",
  });

  const confirmation = await confirmTransaction(
    provider.connection as any,
    txId
  );
  console.log("confirmation", confirmation);

  console.log();
  console.log(
    "LookUpTable",
    `https://explorer.solana.com/tx/${txId}?cluster=custom&customUrl=http://localhost:8899`
  );
  console.log();
};

run();
