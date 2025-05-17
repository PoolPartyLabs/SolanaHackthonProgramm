import {
  CLMM_PROGRAM_ID,
  DEVNET_PROGRAM_ID,
  POOL_TICK_ARRAY_BITMAP_SEED,
} from "@raydium-io/raydium-sdk-v2";
import { LAMPORTS_PER_SOL, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";

const VALID_PROGRAM_ID = new Set([
  CLMM_PROGRAM_ID.toBase58(),
  DEVNET_PROGRAM_ID.CLMM.toBase58(),
]);

export const isValidClmm = (id: string) => VALID_PROGRAM_ID.has(id);

export const printSimulateInfo = () => {
  console.log(
    "you can paste simulate tx string here: https://explorer.solana.com/tx/inspector and click simulate to check transaction status"
  );
  console.log(
    "if tx simulate successful but did not went through successfully after running execute(xxx), usually means your txs were expired, try set up higher priority fees"
  );
  console.log(
    "strongly suggest use paid rpcs would get you better performance"
  );
};

export const airdrop = async (
  connection: anchor.web3.Connection,
  address: PublicKey,
  amount = 100 /** max testnet/devnet 100 SOL */ * LAMPORTS_PER_SOL
) => {
  const signature = await connection.requestAirdrop(address, amount);
  const latestBlockhash = await connection.getLatestBlockhash();
  await connection.confirmTransaction(
    {
      signature,
      blockhash: latestBlockhash.blockhash,
      lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
    },
    "confirmed"
  );
  // await connection.confirmTransaction(
  //   await connection.requestAirdrop(address, amount),
  //   "confirmed"
  // );
};

export const sleep = (ms: number) => {
  return new Promise((resolve) => setTimeout(resolve, ms));
};

export const getNftMetadataAddress = async (
  nftMint: PublicKey
): Promise<[PublicKey, number]> => {
  const [address, bump] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("metadata"),
      new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").toBuffer(),
      nftMint.toBuffer(),
    ],
    new PublicKey("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s")
  );
  return [address, bump];
};

export const getTickArrayBitmapAddress = async (
  pool: PublicKey,
  programId: PublicKey
): Promise<[PublicKey, number]> => {
  const [address, bump] = await PublicKey.findProgramAddress(
    [POOL_TICK_ARRAY_BITMAP_SEED, pool.toBuffer()],
    programId
  );
  return [address, bump];
};

// @note raydium uses that way https://github.com/raydium-io/raydium-cpi-example/blob/master/cpmm-cpi/tests/utils/pda.ts
// https://github.com/raydium-io/raydium-sdk-V2/blob/master/src/raydium/clmm/utils/pda.ts
export const i32ToBytes = (num: number) => {
  const arr = new ArrayBuffer(4);
  const view = new DataView(arr);
  view.setInt32(0, num, false);
  return new Uint8Array(arr);
};
