import {
  AddressLookupTableAccount,
  ComputeBudgetProgram,
  Connection,
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  Signer,
  SystemProgram,
  Transaction,
  TransactionInstruction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import dotenv from "dotenv";
import { _owner, devConfigs, initSdk, txVersion } from "./config";
import { Raydium } from "@raydium-io/raydium-sdk-v2";
import {
  ACCOUNT_SIZE,
  createAssociatedTokenAccount,
  createCloseAccountInstruction,
  createInitializeAccountInstruction,
  createMint,
  createTransferInstruction,
  getAssociatedTokenAddress,
  getAssociatedTokenAddressSync,
  mintTo,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import Decimal from "decimal.js";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import { getSimulationComputeUnits } from "@solana-developers/helpers";
import { createPosition } from "./createPosition";
import { airdrop, sleep } from "./utils";

dotenv.config();

anchor.setProvider(anchor.AnchorProvider.env());

const CLMM_PROGRAM_ID = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";
const CLMM_PROGRAM_ID_DEVNET = "devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH";

// const raydium = await initSdk();
// // RAY-USDC
// const pool1 = '61R1ndXxvsWXXkWSyNkCxnzwd3zUNB8Q2ibmkiLPC8ht'
// // SOL-USDC
// const pool2 = '8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj'
// 4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU USDC Devnet https://faucet.circle.com/

let feePayer: Keypair;
let raydium: Raydium;

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const run = async () => {
  feePayer = _owner;
  await airdrop(
    provider.connection,
    new PublicKey(process.env.WALLET_PUBLIC_KEY), 
    50_000 * LAMPORTS_PER_SOL
  );
  await airdrop(
    provider.connection,
    feePayer.publicKey,
    50_000 * LAMPORTS_PER_SOL
  );
  provider.connection.getBalance(feePayer.publicKey).then((balance) => {
    console.log(`After setup The lamport balance is ${balance}`);
    console.log();
  });

  raydium = await initSdk({ owner: feePayer, loadToken: true });

  // await _createMints();

  let mintA = new PublicKey("B51gXnF3mWxTJ9nTSyBuWH7QHKMa29QcqMEXj4t5FVnH");
  let mintB = new PublicKey("4dD6TpR8FHfYjUH3PDrKtzWwrR9ssYhLMj1HudruWzeQ");
  if (mintA.toBase58() > mintB.toBase58()) {
    const temp = mintA;
    mintA = mintB;
    mintB = temp;
  }

  const poolIdA = await createPool(
    raydium,
    NATIVE_MINT.toBase58(),
    mintA.toBase58(),
    true
  ); // WSOL - mintA => poolIdA J4XJekUt9aXzhrqWDpm5kujo1fTkkSmmtrHedixswks1

  const poolIdB = await createPool(
    raydium,
    NATIVE_MINT.toBase58(),
    mintB.toBase58(),
    true
  ); //// WSOL - mintB => poolIdB GhCPpMsZChcuPmmfxCMJrWz2CtL5xBzSTFQLPrMzpWsf

  const poolIdC = await createPool(
    raydium,
    mintA.toBase58(),
    mintB.toBase58(),
    true
  ); //// mintA - mintB => poolIdC RHkbPaJvxRzLN6hvAgqHTQEU1qdqurHR7iYrGwJ11LD

  //// tGdnGNF8yN2hh7UYd6YD8G9A99Uhe5ngWxHVx5YT99M
  await createPosition(raydium, poolIdA).catch(console.error);
  //// DPm4LWGkQcjTYmhwAVJW9y449jHsx78bWCsXQFYPiysP
  await createPosition(raydium, poolIdB).catch(console.error);

  console.log("done");
  process.exit();
};

const _createMints = async () => {
  const mintA = await _createMintTo(provider.connection as any, feePayer);
  console.log(mintA);
  const mintB = await _createMintTo(provider.connection as any, feePayer);
  console.log(mintB);

  let ata;
  try {
    ata = await createAssociatedTokenAccount(
      provider.connection as any,
      feePayer,
      NATIVE_MINT,
      feePayer.publicKey,
      {
        commitment: "confirmed",
      },
      TOKEN_PROGRAM_ID,
      ASSOCIATED_PROGRAM_ID
      // true
    );
  } catch (e) {
    ata = await getAssociatedTokenAddress(
      NATIVE_MINT, // mint
      feePayer.publicKey // owner
    );
  }
  let auxAccount = Keypair.generate();

  let amount = 5_000 * LAMPORTS_PER_SOL; /* Wrapped SOL's decimals is 9 */

  let tx = new Transaction().add(
    // create token account
    SystemProgram.createAccount({
      fromPubkey: feePayer.publicKey,
      newAccountPubkey: auxAccount.publicKey,
      space: ACCOUNT_SIZE,
      lamports:
        (await getMinimumBalanceForRentExemptAccount(provider.connection)) +
        amount, // rent + amount
      programId: TOKEN_PROGRAM_ID,
    }),
    // init token account
    createInitializeAccountInstruction(
      auxAccount.publicKey,
      NATIVE_MINT,
      feePayer.publicKey
    ),
    // transfer WSOL
    createTransferInstruction(
      auxAccount.publicKey,
      ata,
      feePayer.publicKey,
      amount
    ),
    // close aux account
    createCloseAccountInstruction(
      auxAccount.publicKey,
      feePayer.publicKey,
      feePayer.publicKey
    )
  );

  console.log(
    `SOL -> WSOL txhash: ${await sendAndConfirmTransaction(
      provider.connection as any,
      tx,
      [feePayer, auxAccount, feePayer]
    )}`
  );

  // _mintTo(provider.connection, feePayer, mintA, 100000 * LAMPORTS_PER_SOL);
  // _mintTo(provider.connection, feePayer, mintB, 100000 * LAMPORTS_PER_SOL);

  return {
    mintA: new PublicKey(mintA.toBase58()),
    mintB: new PublicKey(mintB.toBase58()),
  };
};

const _createMintTo = async (
  connection: Connection,
  to: Keypair
): Promise<PublicKey> => {
  const mint = await createMint(
    connection,
    feePayer,
    feePayer.publicKey,
    null,
    9,
    undefined,
    {
      commitment: "confirmed",
    }
  );

  const toAta = await createAssociatedTokenAccount(
    connection,
    feePayer,
    mint,
    to.publicKey,
    {
      commitment: "confirmed",
    }
  );

  const mintAmount = 1_000_000 * LAMPORTS_PER_SOL;
  await mintTo(
    connection,
    feePayer,
    mint,
    toAta,
    feePayer.publicKey,
    mintAmount,
    undefined,
    {
      commitment: "confirmed",
    }
  );

  return mint;
};

const _mintTo = async (
  connection: Connection,
  to: Keypair,
  mint: PublicKey,
  amount: number
): Promise<PublicKey> => {
  const toAta = getAssociatedTokenAddressSync(mint, to.publicKey);

  await mintTo(connection, to, mint, toAta, to.publicKey, amount, undefined, {
    commitment: "confirmed",
  });

  return mint;
};

const createPool = async (
  raydium: Raydium,
  mintA: string,
  mintB: string,
  dumpMainnet: boolean = false
) => {
  // const raydium = await initSdk({ loadToken: true });
  // you can call sdk api to get mint info or paste mint info from api: https://api-v3.raydium.io/mint/list
  const mint1 = await raydium.token.getTokenInfo(mintA);
  const mint2 = await raydium.token.getTokenInfo(mintB);
  let clmmConfigs;
  let programId;
  if (dumpMainnet) {
    clmmConfigs = await raydium.api.getClmmConfigs();
    programId = new PublicKey(CLMM_PROGRAM_ID);
  } else {
    programId = new PublicKey(CLMM_PROGRAM_ID_DEVNET);
    clmmConfigs = devConfigs; // devnet configs
  }
  console.log("clmmConfigs", clmmConfigs[0]);
  const createPoolTx = await raydium.clmm.createPool({
    programId,
    mint1,
    mint2,
    ammConfig: {
      ...clmmConfigs[0],
      id: new PublicKey(clmmConfigs[0].id),
      fundOwner: "",
      description: "",
    },
    initialPrice: new Decimal(1),
    txVersion,
  });
  // don't want to wait confirm, set sendAndConfirm to false or don't pass any params to execute
  const poolId = createPoolTx.extInfo.mockPoolInfo.id;
  console.log("poolId... ", poolId);
  try {
    const { txId } = await createPoolTx.execute({ sendAndConfirm: true });
    console.log("clmm pool created:", {
      txId: `https://explorer.solana.com/tx/${txId}`,
    });
    return poolId;
  } catch (e) {
    console.error("createPool error", e);
    return poolId;
  }
};

const getMinimumBalanceForRentExemptAccount = async (
  connection: anchor.web3.Connection
): Promise<number> => {
  return await connection.getMinimumBalanceForRentExemption(ACCOUNT_SIZE);
};

const buildOptimalTransaction = async (
  connection: Connection,
  instructions: Array<TransactionInstruction>,
  signer: Signer,
  lookupTables: Array<AddressLookupTableAccount>
) => {
  const [microLamports, units, recentBlockhash] = await Promise.all([
    100 /* Get optimal priority fees - https://solana.com/developers/guides/advanced/how-to-use-priority-fees*/,
    getSimulationComputeUnits(
      connection as any, // Cast to any to bypass type checking between different versions
      instructions,
      signer.publicKey,
      lookupTables
    ),
    connection.getLatestBlockhash(),
  ]);

  instructions.unshift(
    ComputeBudgetProgram.setComputeUnitPrice({ microLamports })
  );
  if (units) {
    // probably should add some margin of error to units
    instructions.unshift(ComputeBudgetProgram.setComputeUnitLimit({ units }));
  }
  return {
    transaction: new VersionedTransaction(
      new TransactionMessage({
        instructions,
        recentBlockhash: recentBlockhash.blockhash,
        payerKey: signer.publicKey,
      }).compileToV0Message(lookupTables)
    ),
    recentBlockhash,
  };
};

run();
