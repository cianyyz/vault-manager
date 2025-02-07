import * as anchor from "@coral-xyz/anchor";
import { Program, AnchorProvider, Wallet as AnchorWallet } from "@coral-xyz/anchor";
import { 
  TOKEN_PROGRAM_ID, 
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAssociatedTokenAddress,
  getAccount,
} from "@solana/spl-token";

import { expect } from "chai";
import { Keypair, Connection, PublicKey } from "@solana/web3.js";
import { Vault } from "../target/types/vault";
import { jupiter } from "@jup-ag/core";

import {
  COMMITMENT,
  createMint,
  createTokenAccount,
} from "./utils";

describe("jupiter", () => {
  // Use the same test wallet setup as other tests
  const TEST_PROVIDER_URL = "http://localhost:8899";
  const TEST_WALLET_SECRET = [171,47,220,229,16,25,41,67,249,72,87,200,99,166,155,51,227,166,151,173,73,247,62,43,121,185,218,247,54,154,12,174,176,136,16,247,145,71,131,112,92,104,49,155,204,211,96,225,184,95,61,41,136,83,9,18,137,122,214,38,247,37,158,162];
  
  const connection = new Connection(TEST_PROVIDER_URL, "confirmed");
  const testWallet = Keypair.fromSecretKey(new Uint8Array(TEST_WALLET_SECRET));
  const provider = new AnchorProvider(connection, new AnchorWallet(testWallet), {commitment: "confirmed"});
  anchor.setProvider(provider);

  const program = anchor.workspace.Vault as Program<Vault>;

  // USDC on mainnet
  const USDC_MINT = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
  
  it("executes a Jupiter swap through the vault", async () => {
    try {
      const owner = testWallet.publicKey;
      
      // Create test tokens and accounts
      const tokenAMint = await createMint(provider);
      const tokenBMint = await createMint(provider);
      
      const ownerTokenAAccount = await createTokenAccount(
        provider,
        owner,
        tokenAMint,
        1000000 // Initial balance
      );
      
      const ownerTokenBAccount = await createTokenAccount(
        provider,
        owner,
        tokenBMint
      );

      // Setup Jupiter instance
      const jupiterInstance = await jupiter.init({
        connection,
        cluster: "mainnet",
        user: owner,
      });

      // Get routes
      const routes = await jupiterInstance.computeRoutes({
        inputMint: tokenAMint,
        outputMint: tokenBMint,
        amount: 100000, // Amount to swap
        slippageBps: 50, // 0.5% slippage
      });

      const bestRoute = routes.routesInfos[0];
      
      // Execute swap through vault
      const swapResult = await program.methods
        .proxyJupiterSwap(
          new anchor.BN(100000), // amount_in
          new anchor.BN(0), // minimum_amount_out
          bestRoute.swapInfo // route
        )
        .accounts({
          jupiterProgram: jupiter.JUPITER_PROGRAM_ID,
          authority: owner,
          sourceTokenAccount: ownerTokenAAccount,
          destinationTokenAccount: ownerTokenBAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc(COMMITMENT);

      // Verify the swap results
      const destinationAccount = await getAccount(
        connection,
        ownerTokenBAccount,
        COMMITMENT.commitment
      );

      expect(Number(destinationAccount.amount)).to.be.greaterThan(0);
      
    } catch (error) {
      console.error(error);
      throw error;
    }
  });

  it("fails when non-owner tries to execute swap", async () => {
    // Create a different wallet
    const nonOwnerWallet = Keypair.generate();
    
    try {
      const tokenAMint = await createMint(provider);
      const tokenBMint = await createMint(provider);
      
      const nonOwnerTokenAAccount = await createTokenAccount(
        provider,
        nonOwnerWallet.publicKey,
        tokenAMint,
        1000000
      );
      
      const nonOwnerTokenBAccount = await createTokenAccount(
        provider,
        nonOwnerWallet.publicKey,
        tokenBMint
      );

      // Attempt swap with non-owner
      await program.methods
        .proxyJupiterSwap(
          new anchor.BN(100000),
          new anchor.BN(0),
          [] // Empty route for this test
        )
        .accounts({
          jupiterProgram: jupiter.JUPITER_PROGRAM_ID,
          authority: nonOwnerWallet.publicKey,
          sourceTokenAccount: nonOwnerTokenAAccount,
          destinationTokenAccount: nonOwnerTokenBAccount,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc(COMMITMENT);

      // Should not reach here
      expect.fail("Expected transaction to fail");
    } catch (error) {
      // Verify it failed with unauthorized access error
      expect(error).to.be.an("error");
      expect(error.toString()).to.include("UnauthorizedAccess");
    }
  });
}); 