import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Ovwigho } from "../target/types/ovwigho";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  sendAndConfirmTransaction,
  Transaction,
} from "@solana/web3.js";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { findTreeConfigPda } from "@metaplex-foundation/mpl-bubblegum";
import { publicKey } from "@metaplex-foundation/umi";
import {
  ValidDepthSizePair,
  getConcurrentMerkleTreeAccountSize,
} from "@solana/spl-account-compression";

describe("ovwigho", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.ovwigho as Program<Ovwigho>;
  const umi = createUmi(provider.connection.rpcEndpoint);
  const MPL_ACCOUNT_COMPRESSION_PROGRAM_ID = new PublicKey(
    "mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW"
  );

  let authority = provider.wallet;
  const wallet = provider.wallet as anchor.Wallet;

  const CnftCollection = Keypair.generate();
  const nftCollection = Keypair.generate();
  const emptyMerkleTree = Keypair.generate();

  let configPda: PublicKey;
  let treeConfigPda: PublicKey;

  before(async () => {
    configPda = PublicKey.findProgramAddressSync(
      [Buffer.from("config"), authority.publicKey.toBuffer()],
      program.programId
    )[0];

    const treeConfig = findTreeConfigPda(umi, {
      merkleTree: publicKey(emptyMerkleTree.publicKey.toBase58()),
    })[0];

    treeConfigPda = new PublicKey(treeConfig);
  });

  describe("Initialize Config", () => {
    it("initializes the config", async () => {
      const maxDepth = 14;
      const maxBufferSize = 64;
      const canopyDepth = maxDepth - 5;

      const requiredTreeSpace = getConcurrentMerkleTreeAccountSize(
        maxDepth,
        maxBufferSize,
        canopyDepth ?? 0
      );

      let allocTreeIx = SystemProgram.createAccount({
        fromPubkey: provider.wallet.publicKey,
        lamports: await provider.connection.getMinimumBalanceForRentExemption(
          requiredTreeSpace
        ),
        newAccountPubkey: emptyMerkleTree.publicKey,
        programId: MPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
        space: requiredTreeSpace,
      });

      const sig = await sendAndConfirmTransaction(
        provider.connection,
        new Transaction().add(allocTreeIx),
        [wallet.payer, emptyMerkleTree]
      );

      console.log("allocated merkle tree signature: ", sig);

      try {
        const sig = await program.methods
          .initialize(
            14,
            64,
            {
              name: "test cNFT",
              uri: "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/cnft%20metadata.json",
            },
            {
              name: "test NFT",
              uri: "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/nft%20metadata.json",
            }
          )
          .accounts({
            authority: wallet.publicKey,
            cnftCollection: CnftCollection.publicKey,
            nftCollection: nftCollection.publicKey,
            treeConfig: treeConfigPda,
            merkleTree: emptyMerkleTree.publicKey,
          })
          .signers([CnftCollection, nftCollection])
          .rpc();

        console.log("config account created");
        console.log("merkle tree initialized");
        console.log("cnft collection created");
        console.log("nft collection created");
        console.log("transaction signature", sig);
      } catch (error: any) {
        console.error(`something went wrong: ${error}`);
        if (error.logs && Array.isArray(error.logs)) {
          console.log("Transaction Logs:");
          error.logs.forEach((log: string) => console.log(log));
        } else {
          console.log("No logs available in the error.");
        }
        throw error;
      }
    });
  });

  describe("Mint cNFT", () => {
    it("mints a cnft", async () => {});
  });
});
