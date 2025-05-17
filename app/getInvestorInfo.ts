import { Keypair, PublicKey } from "@solana/web3.js";
import * as anchor from "@coral-xyz/anchor";
import { BN, Program } from "@coral-xyz/anchor";
import dotenv from "dotenv";
import { _owner, initSdk, initSdkWithoutOwner } from "./config";
import {
  ApiV3PoolInfoConcentratedItem,
  getPdaPersonalPositionAddress,
  PositionUtils,
  Raydium,
  TickUtils,
  PositionInfoLayout,
  U64_IGNORE_RANGE,
  TickArrayLayout,
} from "@raydium-io/raydium-sdk-v2";

import Decimal from "decimal.js";
import { PoolParty } from "../target/types/pool_party";
import { getAssociatedTokenAddressSync } from "@solana/spl-token";
import { CLMM_PROGRAM_ID, MINT_A, MINT_B, NFT, POOL_NAME } from "./constants";
import { pools, protocolPDAs } from "./helpers";

dotenv.config();

let feePayer: Keypair;

let manager: Keypair;

let investor: Keypair;

let raydium: Raydium;

const program = anchor.workspace.PoolParty as Program<PoolParty>;

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const run = async () => {
  manager = _owner;
  provider.connection.getBalance(manager.publicKey).then((balance) => {
    console.log(`After createPoolPosition The lamport balance is ${balance}`);
    console.log();
  });

  investor = manager;

  raydium = await initSdkWithoutOwner({ loadToken: true });

  const { mintA, mintB } = pools();

  const data = await fetchInvestorData(mintA, mintB, NFT, investor.publicKey);
  console.log("data", data);
};

run();

export async function fetchInvestorData(
  mintA: PublicKey,
  mintB: PublicKey,
  NFT: PublicKey, // NFT address
  investorPublicKey: PublicKey, // User wallet address,
  poolName: string = POOL_NAME
) {
  try {
    const { poolPositionConfig, poolPosition } = protocolPDAs(
      program,
      poolName
    );
    const [investorAccountPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("investor:"),
        poolPositionConfig.toBuffer(),
        investorPublicKey.toBuffer(),
      ],
      program.programId
    );

    const investorAccount = await program.account.investor.fetch(
      investorAccountPda
    );

    const positionPubKey = getPdaPersonalPositionAddress(
      new PublicKey(CLMM_PROGRAM_ID),
      NFT
    ).publicKey;

    const positionAccountInfo = await raydium.connection.getAccountInfo(
      positionPubKey
    );
    const position = PositionInfoLayout.decode(positionAccountInfo!.data);

    let poolInfo: ApiV3PoolInfoConcentratedItem;
    if (raydium.cluster === "mainnet") {
      poolInfo = (
        await raydium.api.fetchPoolById({ ids: position.poolId.toBase58() })
      )[0] as ApiV3PoolInfoConcentratedItem;
    } else {
      const data = await raydium.clmm.getPoolInfoFromRpc(
        position.poolId.toBase58()
      );
      poolInfo = data.poolInfo;
    }

    const epochInfo = await raydium.connection.getEpochInfo();
    const { amountA, amountB } = PositionUtils.getAmountsFromLiquidity({
      poolInfo,
      ownerPosition: position,
      liquidity: investorAccount.liquidity,
      slippage: 0,
      add: false,
      epochInfo,
    });
    const [pooledAmount0, pooledAmount1] = [
      new Decimal(amountA.amount.toString()).div(10 ** poolInfo.mintA.decimals),
      new Decimal(amountB.amount.toString()).div(10 ** poolInfo.mintB.decimals),
    ];

    const { amountA: positionAmountA, amountB: positionAmountB } =
      PositionUtils.getAmountsFromLiquidity({
        poolInfo,
        ownerPosition: position,
        liquidity: position.liquidity,
        slippage: 0,
        add: false,
        epochInfo,
      });
    const [positionPooledAmount0, positionPooledAmount1] = [
      new Decimal(positionAmountA.amount.toString()).div(
        10 ** poolInfo.mintA.decimals
      ),
      new Decimal(positionAmountB.amount.toString()).div(
        10 ** poolInfo.mintB.decimals
      ),
    ];

    // const [feesVault0Pda] = PublicKey.findProgramAddressSync(
    //   [Buffer.from("pool_pos_fees_vault_0:"), poolPosition.toBuffer()],
    //   program.programId
    // );

    // const [feesVault1Pda] = PublicKey.findProgramAddressSync(
    //   [Buffer.from("pool_pos_fees_vault_1:"), poolPosition.toBuffer()],
    //   program.programId
    // );
    // Get Protocol Pool Position token fees balances
    // const feeAccount0Balance = await provider.connection.getTokenAccountBalance(
    //   feesVault0Pda
    // );
    // const feeAccount1Balance = await provider.connection.getTokenAccountBalance(
    //   feesVault1Pda
    // );

    const poolPositionAccount = await program.account.poolPosition.fetch(
      poolPosition
    );

    const [tickLowerArrayAddress, tickUpperArrayAddress] = [
      TickUtils.getTickArrayAddressByTick(
        new PublicKey(poolInfo.programId),
        new PublicKey(poolInfo.id),
        position.tickLower,
        poolInfo.config.tickSpacing
      ),
      TickUtils.getTickArrayAddressByTick(
        new PublicKey(poolInfo.programId),
        new PublicKey(poolInfo.id),
        position.tickUpper,
        poolInfo.config.tickSpacing
      ),
    ];

    const tickArrayRes = await raydium.connection.getMultipleAccountsInfo([
      tickLowerArrayAddress,
      tickUpperArrayAddress,
    ]);
    if (!tickArrayRes[0] || !tickArrayRes[1]) {
      throw new Error("tick data not found");
    }
    const tickArrayLower = TickArrayLayout.decode(tickArrayRes[0].data);
    const tickArrayUpper = TickArrayLayout.decode(tickArrayRes[1].data);
    const tickLowerState =
      tickArrayLower.ticks[
        TickUtils.getTickOffsetInArray(
          position.tickLower,
          poolInfo.config.tickSpacing
        )
      ];
    const tickUpperState =
      tickArrayUpper.ticks[
        TickUtils.getTickOffsetInArray(
          position.tickUpper,
          poolInfo.config.tickSpacing
        )
      ];
    const rpcPoolData = await raydium.clmm.getRpcClmmPoolInfo({
      poolId: position.poolId,
    });
    const tokenFees = PositionUtils.GetPositionFeesV2(
      rpcPoolData,
      position,
      tickLowerState,
      tickUpperState
    );
    const [tokenFeeAmountA, tokenFeeAmountB] = [
      tokenFees.tokenFeeAmountA.gte(new BN(0)) &&
      tokenFees.tokenFeeAmountA.lt(U64_IGNORE_RANGE)
        ? tokenFees.tokenFeeAmountA
        : new BN(0),
      tokenFees.tokenFeeAmountB.gte(new BN(0)) &&
      tokenFees.tokenFeeAmountB.lt(U64_IGNORE_RANGE)
        ? tokenFees.tokenFeeAmountB
        : new BN(0),
    ];

    const feesIndex0 = tokenFeeAmountA
      .mul(new BN(1e10))
      .div(position.liquidity)
      .add(poolPositionAccount.feesIndex0);

    const investorFees0 = investorAccount.liquidity
      .mul(feesIndex0.sub(investorAccount.feesIndex0))
      .div(new BN(1e10));

    const feesIndex1 = tokenFeeAmountB
      .mul(new BN(1e10))
      .div(position.liquidity)
      .add(poolPositionAccount.feesIndex1);

    const investorFees1 = investorAccount.liquidity
      .mul(feesIndex1.sub(investorAccount.feesIndex1))
      .div(new BN(1e10));

    const tokenA = getAssociatedTokenAddressSync(
      mintA,
      investor.publicKey,
      true
    );

    const token0 = getAssociatedTokenAddressSync(
      mintB,
      investor.publicKey,
      true
    );
    // Get token balances
    const token0Balance = await provider.connection.getTokenAccountBalance(
      tokenA
    );
    const token1Balance = await provider.connection.getTokenAccountBalance(
      token0
    );

    return {
      investorAccount: {
        ...investorAccount,
        authority: investorPublicKey.toBase58(),
        poolPositionKey: poolPosition.toBase58(),
      },
      position: {
        ...position,
        poolId: position.poolId.toBase58(),
        nftMint: position.nftMint.toBase58(),
        pooledAmount0: positionPooledAmount0,
        pooledAmount1: positionPooledAmount1,
      },
      pooledAmount0,
      pooledAmount1,
      owedFees0: new Decimal(investorFees0.toString())
        .div(10 ** poolInfo.mintA.decimals)
        .toString(),
      owedFees1: new Decimal(investorFees1.toString())
        .div(10 ** poolInfo.mintB.decimals)
        .toString(),
      token0Balance: token0Balance.value.uiAmount,
      token1Balance: token1Balance.value.uiAmount,
    };
  } catch (error) {
    console.error("Error fetching investor data:", error);
    throw error;
  }
}
