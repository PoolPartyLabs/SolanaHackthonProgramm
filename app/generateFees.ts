import { Keypair, LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import dotenv from "dotenv";
import { _owner, initSdk } from "./config";
import { Raydium } from "@raydium-io/raydium-sdk-v2";
import { swap } from "./swap";
import { airdrop, sleep } from "./utils";
import { fetchPositionInfo } from "./fetchPositionInfo";
import { MINT_A, MINT_B, NFT, POOL_ID } from "./constants";

dotenv.config();

anchor.setProvider(anchor.AnchorProvider.env());

let feePayer: Keypair;
let raydium: Raydium;

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const run = async () => {
  feePayer = _owner;
  await airdrop(provider.connection, feePayer.publicKey);
  provider.connection.getBalance(feePayer.publicKey).then((balance) => {
    console.log(`After generateFees The lamport balance is ${balance}`);
    console.log();
  });

  raydium = await initSdk({ owner: feePayer, loadToken: true });

  let mintA = MINT_A;
  let mintB = MINT_B;

  const poolIdC = POOL_ID.toBase58();

  for (let i = 0; i < 200; i++) {
    await swap(
      raydium,
      poolIdC,
      mintA.toBase58(),
      new anchor.BN(50 * LAMPORTS_PER_SOL)
    ).catch(console.error);

    sleep(1000);
    await swap(
      raydium,
      poolIdC,
      mintB.toBase58(),
      new anchor.BN(40 * LAMPORTS_PER_SOL)
    ).catch(console.error);
  }
  await fetchPositionInfo(raydium, NFT.toBase58()).catch(console.error);

  console.log("done");
  process.exit();
};

run();
