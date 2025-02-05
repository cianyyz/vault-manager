import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAccount,
  getMint 
} from "@solana/spl-token";

import { expect } from "chai";
import web3, { Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { Vault } from "../target/types/vault";

import {
  COMMITMENT,
  PDAAccounts,
  ParsedTokenTransfer,
  createMint,
  createTokenAccount,
  getPDAs,
} from "./utils";

describe("withdraw", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const { connection } = provider;

  const program = anchor.workspace.Vault as Program<Vault>;

  it("withdraws USDC from vault and burns share tokens", async () => {
    try {
      const owner = provider.wallet.publicKey;
      const mint = await createMint(provider);
      const ownerTokenAccount = await createTokenAccount(
        provider,
        provider.wallet.publicKey,
        mint,
        100_000 * LAMPORTS_PER_SOL
      );

      const { vault, vaultTokenAccount, vaultAuthority } = await getPDAs({
        owner,
        programId: program.programId,
        mint,
      });

      // Initialize vault first
      await program.methods
        .initialize(new anchor.BN(10))
        .accounts({
          vault,
          owner,
          mint,
          ownerTokenAccount,
          vaultAuthority,
          vaultTokenAccount,
          systemProgram: web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc(COMMITMENT);

      // Create share token accounts
      const withdrawerShareAccount = await getAssociatedTokenAddress(
        mint,
        owner,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      const ownerShareAccount = await getAssociatedTokenAddress(
        mint,
        owner,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      // Deposit first to get some share tokens
      await program.methods
        .depositUsdc(new anchor.BN(10))
        .accounts({
          depositor: owner,
          vault,
          vaultUsdc: vaultTokenAccount,
          depositorUsdc: ownerTokenAccount,
          shareMint: mint,
          depositorShareAccount: withdrawerShareAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
          rent: web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc(COMMITMENT);

      // Now withdraw
      const withdrawTransaction = await program.methods
        .withdrawUsdc(new anchor.BN(5))
        .accounts({
          withdrawer: owner,
          vault,
          vaultOwner: owner,
          ownerShareAccount,
          vaultUsdc: vaultTokenAccount,
          withdrawerUsdc: ownerTokenAccount,
          shareMint: mint,
          withdrawerShareAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc(COMMITMENT);

      const tx = await connection.getParsedTransaction(
        withdrawTransaction,
        COMMITMENT
      );

      // Check USDC transfer
      const transferIx: any = tx.meta.innerInstructions[0].instructions.find(
        (ix) =>
          (ix as any).parsed.type === "transfer" &&
          ix.programId.toBase58() == TOKEN_PROGRAM_ID.toBase58()
      );
      const parsedInfo: ParsedTokenTransfer = transferIx.parsed.info;
      
      // Calculate expected USDC amount after fees
      const withdrawAmount = 5;
      const ownerFee = Math.floor((withdrawAmount * 50) / 10000); // 0.5%
      const burnFee = Math.floor((withdrawAmount * 100) / 10000); // 1%
      const expectedUsdcAmount = withdrawAmount - ownerFee - burnFee;

      expect(parsedInfo).eql({
        amount: expectedUsdcAmount.toString(),
        authority: vault.toBase58(),
        destination: ownerTokenAccount.toBase58(),
        source: vaultTokenAccount.toBase58(),
      });

      // Check vault data
      const vaultData = await program.account.vault.fetch(vault);
      expect(vaultData.owner.toBase58()).to.eq(owner.toBase58());
      expect(vaultData.shareMint.toBase58()).to.eq(mint.toBase58());
      expect(vaultData.totalShares.toNumber()).to.eq(10 - burnFee); // Initial shares minus burned shares
      expect(vaultData.bumps.vault).to.not.eql(0);
      expect(vaultData.bumps.vaultAuthority).to.not.eql(0);
      expect(vaultData.bumps.vaultTokenAccount).to.not.eql(0);

      // Check share token balances
      const withdrawerBalance = await getAccount(
        connection,
        withdrawerShareAccount,
        COMMITMENT.commitment,
        TOKEN_PROGRAM_ID
      );
      const ownerBalance = await getAccount(
        connection,
        ownerShareAccount,
        COMMITMENT.commitment,
        TOKEN_PROGRAM_ID
      );

      expect(Number(withdrawerBalance.amount)).to.eq(5 - burnFee);
      expect(Number(ownerBalance.amount)).to.eq(ownerFee);

    } catch (error) {
      console.error(error);
      throw error;
    }
  });
});