import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getMint,
  createSetAuthorityInstruction,
  AuthorityType
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

describe("initialize", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const { connection } = provider;

  const program = anchor.workspace.Vault as Program<Vault>;

  it("Initializes the vault account and deposits into the vault token account", async () => {
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

      // Initialize vault
      const initializeTransaction = await program.methods
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

      // Transfer mint authority to vault
      const transferAuthTx = new web3.Transaction().add(
        createSetAuthorityInstruction(
          mint,
          owner,
          AuthorityType.MintTokens,
          vault,
          [],
          TOKEN_PROGRAM_ID
        )
      );
      await provider.sendAndConfirm(transferAuthTx);

      // Get and verify the initialize transaction
      const tx = await connection.getParsedTransaction(
        initializeTransaction,
        COMMITMENT
      );

      // Ensure that inner transfer succeeded
      const transferIx: any = tx.meta.innerInstructions[0].instructions.find(
        (ix) =>
          (ix as any).parsed.type === "transfer" &&
          ix.programId.toBase58() == TOKEN_PROGRAM_ID.toBase58()
      );
      const parsedInfo: ParsedTokenTransfer = transferIx.parsed.info;
      expect(parsedInfo).eql({
        amount: "10",
        authority: owner.toBase58(),
        destination: vaultTokenAccount.toBase58(),
        source: ownerTokenAccount.toBase58(),
      });

      // Check vault data
      const vaultData = await program.account.vault.fetch(vault);
      //console.log(vaultData);
      expect(vaultData.owner.toBase58()).to.eq(owner.toBase58());
      expect(vaultData.shareMint.toBase58()).to.eq(mint.toBase58());
      expect(vaultData.totalShares.toNumber()).to.eq(0);
      expect(vaultData.currentPosition).to.eq(null);
      expect(vaultData.currentWhirlpool).to.eq(null);
      expect(vaultData.whitelistedTokens).to.eql([]);
      expect(vaultData.bumps.vault).to.not.eql(0);
      expect(vaultData.bumps.vaultAuthority).to.not.eql(0);
      expect(vaultData.bumps.vaultTokenAccount).to.not.eql(0);

      // Verify mint authority transfer
      const mintInfo = await getMint(
        connection,
        mint,
        COMMITMENT.commitment,
        TOKEN_PROGRAM_ID
      );
      expect(mintInfo.mintAuthority.toBase58()).to.eq(vault.toBase58());
    } catch (error) {
      console.error(error);
      throw error;
    }
  });
});