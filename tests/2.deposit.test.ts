import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Wallet as AnchorWallet } from "@coral-xyz/anchor";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAccount,
  getMint,
  createSetAuthorityInstruction,
  AuthorityType
} from "@solana/spl-token";

import { expect } from "chai";
import web3, { Keypair, LAMPORTS_PER_SOL, Connection } from "@solana/web3.js";
import { Vault } from "../target/types/vault";

import {
  COMMITMENT,
  PDAAccounts,
  ParsedTokenTransfer,
  createMint,
  createTokenAccount,
  getPDAs,
} from "./utils";

describe("deposit", () => {
  // Use the same test wallet setup as orca test
  const TEST_PROVIDER_URL = "http://localhost:8899";
  const TEST_WALLET_SECRET = [171,47,220,229,16,25,41,67,249,72,87,200,99,166,155,51,227,166,151,173,73,247,62,43,121,185,218,247,54,154,12,174,176,136,16,247,145,71,131,112,92,104,49,155,204,211,96,225,184,95,61,41,136,83,9,18,137,122,214,38,247,37,158,162];
  
  const connection = new Connection(TEST_PROVIDER_URL, "confirmed");
  const testWallet = Keypair.fromSecretKey(new Uint8Array(TEST_WALLET_SECRET));
  const provider = new AnchorProvider(connection, new AnchorWallet(testWallet), {commitment: "confirmed"});
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;

  it("deposits into the vault token account and updates total shares", async () => {
    try {
      const owner = testWallet.publicKey;  // Use testWallet instead of provider.wallet
      const mint = await createMint(provider);
      const ownerTokenAccount = await createTokenAccount(
        provider,
        testWallet.publicKey,  // Use testWallet
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

      // Verify mint authority transfer
      const mintInfo = await getMint(
        connection,
        mint,
        COMMITMENT.commitment,
        TOKEN_PROGRAM_ID
      );
      expect(mintInfo.mintAuthority.toBase58()).to.eq(vault.toBase58());

      // Create depositor share token account
      const depositorShareAccount = await getAssociatedTokenAddress(
        mint,
        owner,
        false,
        TOKEN_PROGRAM_ID,
        ASSOCIATED_TOKEN_PROGRAM_ID
      );

      // Now deposit
      const depositTransaction = await program.methods
        .depositUsdc(new anchor.BN(5))
        .accounts({
          depositor: owner,
          vault,
          vaultUsdc: vaultTokenAccount,
          depositorUsdc: ownerTokenAccount,
          shareMint: mint,
          depositorShareAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: web3.SystemProgram.programId,
          rent: web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc(COMMITMENT);

      const tx = await connection.getParsedTransaction(
        depositTransaction,
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
        amount: "5",
        authority: owner.toBase58(),
        destination: vaultTokenAccount.toBase58(),
        source: ownerTokenAccount.toBase58(),
      });

      // Check vault data
      const vaultData = await program.account.vault.fetch(vault);
      expect(vaultData.owner.toBase58()).to.eq(owner.toBase58());
      expect(vaultData.shareMint.toBase58()).to.eq(mint.toBase58());
      expect(vaultData.totalShares.toNumber()).to.eq(5); // First deposit should be 1:1
      expect(vaultData.bumps.vault).to.not.eql(0);
      expect(vaultData.bumps.vaultAuthority).to.not.eql(0);
      expect(vaultData.bumps.vaultTokenAccount).to.not.eql(0);

      // Check share token balance
      const shareBalance = await getAccount(
        connection,
        depositorShareAccount,
        COMMITMENT.commitment,
        TOKEN_PROGRAM_ID
      );
      expect(Number(shareBalance.amount)).to.eq(5);

    } catch (error) {
      console.error(error);
      throw error;
    }
  });
});