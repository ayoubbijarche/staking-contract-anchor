import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Stakingcontract } from "../target/types/stakingcontract";
import { PublicKey, SystemProgram, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, getAssociatedTokenAddress, createAssociatedTokenAccount } from "@solana/spl-token";
import * as token from "@solana/spl-token";

describe("stakingcontract", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Stakingcontract as Program<Stakingcontract>;
  
  const admin = anchor.web3.Keypair.generate();
  const user = provider.wallet.publicKey;
  const poolInfo = anchor.web3.Keypair.generate();
  const userInfo = anchor.web3.Keypair.generate();
  
  let stakingMint: PublicKey;
  let userStakingWallet: PublicKey;
  let adminStakingWallet: PublicKey;

  before(async () => {
    try {
      const airdropSignature = await provider.connection.requestAirdrop(
        admin.publicKey,
        2 * LAMPORTS_PER_SOL
      );
      const latestBlockhash = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        signature: airdropSignature,
        ...latestBlockhash
      });
    } catch (err) {
      console.error("Airdrop failed:", err);
      throw err;
    }

    stakingMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      9
    );

    userStakingWallet = await createAssociatedTokenAccount(
      provider.connection,
      admin,
      stakingMint,
      user
    );

    adminStakingWallet = await createAssociatedTokenAccount(
      provider.connection,
      admin,
      stakingMint,
      admin.publicKey
    );
  });

  it("Is initialized!", async () => {
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    try {
      // Convert slots to BN
      const startSlot = new anchor.BN(100);
      const endSlot = new anchor.BN(1000);

      // Initialize the staking contract
      await program.methods
        .initialize(startSlot, endSlot)
        .accounts({
          admin: admin.publicKey,
          poolInfo: poolInfo.publicKey,
          stakingToken: stakingMint,
          system_program: SystemProgram.programId,
        })
        .signers([admin, poolInfo])
        .rpc();

      // Verify the initialization
      const poolAccount = await program.account.poolInfo.fetch(poolInfo.publicKey);
      console.log("Pool Info:", {
        admin: poolAccount.admin.toString(),
        startSlot: poolAccount.startSlot.toString(),
        endSlot: poolAccount.endSlot.toString(),
        token: poolAccount.token.toString(),
      });
    } catch (err) {
      console.error("Initialization Error:", err);
      throw err;
    }
  });

  it("Can stake tokens", async () => {
    try {
      // Mint tokens to user before staking
      await token.mintTo(
        provider.connection,
        admin,
        stakingMint,
        userStakingWallet,
        admin.publicKey,
        1000
      );

      const stakeAmount = new anchor.BN(100);

      await program.methods
        .stake(stakeAmount)
        .accounts({
          user: user,
          admin: admin.publicKey,
          pool_info: poolInfo.publicKey,
          user_info: userInfo.publicKey,
          user_staking_wallet: userStakingWallet,
          adminStakingWallet,
          stakingToken: stakingMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([userInfo])
        .rpc();

      const userAccount = await program.account.userInfo.fetch(userInfo.publicKey);
      console.log("User Info after stake:", {
        amount: userAccount.amount.toString(),
        debtReward: userAccount.debtReward.toString(),
        depositSlot: userAccount.depositSlot.toString(),
      });
    } catch (err) {
      console.error("Staking Error:", err);
      throw err;
    }
  });

  it("Can claim rewards", async () => {
    try {
      await program.methods
        .claimReward()
        .accounts({
          user: user,
          admin: admin.publicKey,
          pool_info: poolInfo.publicKey,
          user_info: userInfo.publicKey,
          user_staking_wallet: userStakingWallet,
          adminStakingWallet,
          stakingToken: stakingMint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc();

      const userAccount = await program.account.userInfo.fetch(userInfo.publicKey);
      console.log("User Info after claim:", {
        amount: userAccount.amount.toString(),
        debtReward: userAccount.debtReward.toString(),
        depositSlot: userAccount.depositSlot.toString(),
      });
    } catch (err) {
      console.error("Claim Reward Error:", err);
      throw err;
    }
  });
});
