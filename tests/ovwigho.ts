import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Ovwigho } from "../target/types/ovwigho";
import {
  PublicKey,
  Keypair,
  SystemProgram,
  sendAndConfirmTransaction,
  Transaction,
} from "@solana/web3.js";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
  findTreeConfigPda,
  mplBubblegum,
  getAssetWithProof,
  burn,
} from "@metaplex-foundation/mpl-bubblegum";
import { publicKey } from "@metaplex-foundation/umi";
import {
  ValidDepthSizePair,
  getConcurrentMerkleTreeAccountSize,
} from "@solana/spl-account-compression";
import { expect } from "chai";
import {
  DasApiAsset,
  DasApiAssetCreator,
  DasApiAssetGrouping,
  GetAssetProofRpcResponse,
  dasApi,
} from "@metaplex-foundation/digital-asset-standard-api";
import { testPlugins } from "@metaplex-foundation/umi-bundle-tests";
import testKeys from "../test_keys.json";

describe("ovwigho", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.ovwigho as Program<Ovwigho>;
  const umi = createUmi(provider.connection.rpcEndpoint);
  const MPL_ACCOUNT_COMPRESSION_PROGRAM_ID = new PublicKey(
    "mcmt6YrQEMKw8Mw43FmpRLmf7BqRnFMKmAcbxE3xkAW"
  );
  const endpoint =
    "https://devnet.helius-rpc.com/?api-key=91aee762-a5cd-4013-98ef-09e7dfe2da41";

  let authority = provider.wallet;
  const wallet = provider.wallet as anchor.Wallet;

  const CnftCollection = Keypair.generate();
  const nftCollection = Keypair.generate();
  const emptyMerkleTree = Keypair.generate();
  const playerOne = Keypair.fromSecretKey(Uint8Array.from(testKeys.playerOne));
  const playerTwo = Keypair.fromSecretKey(Uint8Array.from(testKeys.playerTwo));

  let configPda: PublicKey;
  let treeConfigPda: PublicKey;
  let playerOneProgressPda: PublicKey;

  before(async () => {
    configPda = PublicKey.findProgramAddressSync(
      [Buffer.from("config"), authority.publicKey.toBuffer()],
      program.programId
    )[0];

    const treeConfig = findTreeConfigPda(umi, {
      merkleTree: publicKey(emptyMerkleTree.publicKey.toBase58()),
    })[0];

    treeConfigPda = new PublicKey(treeConfig);

    playerOneProgressPda = PublicKey.findProgramAddressSync(
      [Buffer.from("player"), playerOne.publicKey.toBuffer()],
      program.programId
    )[0];

    const playerOneBalance = await provider.connection.getBalance(
      playerOne.publicKey
    );
    if (playerOneBalance < 1_000_000_000) {
      const tx = new anchor.web3.Transaction().add(
        SystemProgram.transfer({
          fromPubkey: authority.publicKey,
          toPubkey: playerOne.publicKey,
          lamports: 0.1 * anchor.web3.LAMPORTS_PER_SOL,
        })
      );
      await provider.sendAndConfirm(tx);
    }

    const playerTwoBalance = await provider.connection.getBalance(
      playerTwo.publicKey
    );
    if (playerTwoBalance < 1_000_000_000) {
      const tx = new anchor.web3.Transaction().add(
        SystemProgram.transfer({
          fromPubkey: authority.publicKey,
          toPubkey: playerTwo.publicKey,
          lamports: 0.1 * anchor.web3.LAMPORTS_PER_SOL,
        })
      );
      await provider.sendAndConfirm(tx);
    }
  });

  describe("Initialize Config, Create NFT & cNFT Collections", () => {
    // it("initializes the config", async () => {
    //   const maxDepth = 14;
    //   const maxBufferSize = 64;
    //   const canopyDepth = maxDepth - 5;
    //   const requiredTreeSpace = getConcurrentMerkleTreeAccountSize(
    //     maxDepth,
    //     maxBufferSize,
    //     canopyDepth ?? 0
    //   );
    //   let allocTreeIx = SystemProgram.createAccount({
    //     fromPubkey: provider.wallet.publicKey,
    //     lamports: await provider.connection.getMinimumBalanceForRentExemption(
    //       requiredTreeSpace
    //     ),
    //     newAccountPubkey: emptyMerkleTree.publicKey,
    //     programId: MPL_ACCOUNT_COMPRESSION_PROGRAM_ID,
    //     space: requiredTreeSpace,
    //   });
    //   const sig = await sendAndConfirmTransaction(
    //     provider.connection,
    //     new Transaction().add(allocTreeIx),
    //     [wallet.payer, emptyMerkleTree]
    //   );
    //   console.log("allocated merkle tree signature: ", sig);
    //   try {
    //     const sig = await program.methods
    //       .initialize(
    //         14,
    //         64,
    //         {
    //           name: "test cNFT",
    //           uri: "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/cnft%20metadata.json",
    //         },
    //         {
    //           name: "test NFT",
    //           uri: "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/nft%20metadata.json",
    //         }
    //       )
    //       .accounts({
    //         authority: wallet.publicKey,
    //         cnftCollection: CnftCollection.publicKey,
    //         nftCollection: nftCollection.publicKey,
    //         treeConfig: treeConfigPda,
    //         merkleTree: emptyMerkleTree.publicKey,
    //       })
    //       .signers([CnftCollection, nftCollection])
    //       .rpc();
    //     console.log("config account created");
    //     console.log("merkle tree initialized");
    //     console.log("cnft collection created");
    //     console.log("nft collection created");
    //     console.log("transaction signature", sig);
    //   } catch (error: any) {
    //     console.error(`something went wrong: ${error}`);
    //     if (error.logs && Array.isArray(error.logs)) {
    //       console.log("Transaction Logs:");
    //       error.logs.forEach((log: string) => console.log(log));
    //     } else {
    //       console.log("No logs available in the error.");
    //     }
    //     throw error;
    //   }
    // });
  });

  describe("Mint cNFT", () => {
    // it("mints a cnft for a player", async () => {
    //   let mplCoreCpiSignerKey = new PublicKey(
    //     "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
    //   );
    //   let configAccount = await program.account.config.fetch(configPda);
    //   let initializedCnftCollection = configAccount.cnftCollection;
    //   let initializedMerkleTree = configAccount.merkleTree;
    //   let treeConfig = findTreeConfigPda(umi, {
    //     merkleTree: publicKey(initializedMerkleTree),
    //   })[0];
    //   treeConfigPda = new PublicKey(treeConfig);
    //   try {
    //     const sig = await program.methods
    //       .mintCnft(
    //         "sample cnft",
    //         "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/cnft%20metadata.json",
    //         "SMPL"
    //       )
    //       .accounts({
    //         player: playerOne.publicKey,
    //         authority: wallet.publicKey,
    //         cnftCollection: initializedCnftCollection,
    //         treeConfig: treeConfigPda,
    //         merkleTree: initializedMerkleTree,
    //         mplCoreCpiSigner: mplCoreCpiSignerKey,
    //       })
    //       .signers([playerOne])
    //       .rpc();
    //     console.log("cnft minted");
    //     console.log("transaction signature", sig);
    //   } catch (error: any) {
    //     console.error(`something went wrong: ${error}`);
    //     if (error.logs && Array.isArray(error.logs)) {
    //       console.log("Transaction Logs:");
    //       error.logs.forEach((log: string) => console.log(log));
    //     } else {
    //       console.log("No logs available in the error.");
    //     }
    //     throw error;
    //   }
    // });
    // it("mints a cnft for a different player", async () => {
    //   let mplCoreCpiSignerKey = new PublicKey(
    //     "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
    //   );
    //   let configAccount = await program.account.config.fetch(configPda);
    //   let initializedCnftCollection = configAccount.cnftCollection;
    //   let initializedMerkleTree = configAccount.merkleTree;
    //   let treeConfig = findTreeConfigPda(umi, {
    //     merkleTree: publicKey(initializedMerkleTree),
    //   })[0];
    //   treeConfigPda = new PublicKey(treeConfig);
    //   try {
    //     const sig = await program.methods
    //       .mintCnft(
    //         "sample cnft 1",
    //         "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/cnft%20metadata.json",
    //         "SMPL"
    //       )
    //       .accounts({
    //         player: playerTwo.publicKey,
    //         authority: wallet.publicKey,
    //         cnftCollection: initializedCnftCollection,
    //         treeConfig: treeConfigPda,
    //         merkleTree: initializedMerkleTree,
    //         mplCoreCpiSigner: mplCoreCpiSignerKey,
    //       })
    //       .signers([playerTwo])
    //       .rpc();
    //     console.log("cnft minted");
    //     console.log("transaction signature", sig);
    //   } catch (error: any) {
    //     console.error(`something went wrong: ${error}`);
    //     if (error.logs && Array.isArray(error.logs)) {
    //       console.log("Transaction Logs:");
    //       error.logs.forEach((log: string) => console.log(log));
    //     } else {
    //       console.log("No logs available in the error.");
    //     }
    //     throw error;
    //   }
    // });
    // it("mints another cnft for the same player", async () => {
    //   let mplCoreCpiSignerKey = new PublicKey(
    //     "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
    //   );
    //   let configAccount = await program.account.config.fetch(configPda);
    //   let initializedCnftCollection = configAccount.cnftCollection;
    //   let initializedMerkleTree = configAccount.merkleTree;
    //   let treeConfig = findTreeConfigPda(umi, {
    //     merkleTree: publicKey(initializedMerkleTree),
    //   })[0];
    //   treeConfigPda = new PublicKey(treeConfig);
    //   try {
    //     const sig = await program.methods
    //       .mintCnft(
    //         "sample cnft 2",
    //         "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/cnft%20metadata.json",
    //         "SMPL"
    //       )
    //       .accounts({
    //         player: playerOne.publicKey,
    //         authority: wallet.publicKey,
    //         cnftCollection: initializedCnftCollection,
    //         treeConfig: treeConfigPda,
    //         merkleTree: initializedMerkleTree,
    //         mplCoreCpiSigner: mplCoreCpiSignerKey,
    //       })
    //       .signers([playerOne])
    //       .rpc();
    //     console.log("cnft minted");
    //     console.log("transaction signature", sig);
    //   } catch (error: any) {
    //     console.error(`something went wrong: ${error}`);
    //     if (error.logs && Array.isArray(error.logs)) {
    //       console.log("Transaction Logs:");
    //       error.logs.forEach((log: string) => console.log(log));
    //     } else {
    //       console.log("No logs available in the error.");
    //     }
    //     throw error;
    //   }
    // });
    // it("increments the total number of cnfts minted in the config", async () => {
    //   let configAccount = await program.account.config.fetch(configPda);
    //   expect(configAccount.totalCnftsMinted).to.be.greaterThan(0);
    // });
    // it("fails to mint a cnft with an invalid authority", async () => {
    //   let mplCoreCpiSignerKey = new PublicKey(
    //     "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
    //   );
    //   let configAccount = await program.account.config.fetch(configPda);
    //   let initializedCnftCollection = configAccount.cnftCollection;
    //   let initializedMerkleTree = configAccount.merkleTree;
    //   let treeConfig = findTreeConfigPda(umi, {
    //     merkleTree: publicKey(initializedMerkleTree),
    //   })[0];
    //   treeConfigPda = new PublicKey(treeConfig);
    //   try {
    //     const sig = await program.methods
    //       .mintCnft(
    //         "sample cnft",
    //         "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/cnft%20metadata.json",
    //         "SMPL"
    //       )
    //       .accounts({
    //         player: playerOne.publicKey,
    //         authority: playerTwo.publicKey,
    //         cnftCollection: initializedCnftCollection,
    //         treeConfig: treeConfigPda,
    //         merkleTree: initializedMerkleTree,
    //         mplCoreCpiSigner: mplCoreCpiSignerKey,
    //       })
    //       .signers([playerOne])
    //       .rpc();
    //     expect.fail("should fail with invalid authority");
    //   } catch (error: any) {
    //     expect(error.error?.errorCode?.code || error.message).to.satisfy(
    //       (msg: string) =>
    //         msg.includes("AccountNotInitialized") ||
    //         msg.includes("Signature verification failed")
    //     );
    //   }
    // });
    // it("fails to mint a cnft with an invalid signer", async () => {
    //   let mplCoreCpiSignerKey = new PublicKey(
    //     "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
    //   );
    //   let configAccount = await program.account.config.fetch(configPda);
    //   let initializedCnftCollection = configAccount.cnftCollection;
    //   let initializedMerkleTree = configAccount.merkleTree;
    //   let treeConfig = findTreeConfigPda(umi, {
    //     merkleTree: publicKey(initializedMerkleTree),
    //   })[0];
    //   treeConfigPda = new PublicKey(treeConfig);
    //   try {
    //     const sig = await program.methods
    //       .mintCnft(
    //         "sample cnft",
    //         "https://raw.githubusercontent.com/amxrac/cmd-token/refs/heads/main/cnft%20metadata.json",
    //         "SMPL"
    //       )
    //       .accounts({
    //         player: playerOne.publicKey,
    //         authority: wallet.publicKey,
    //         cnftCollection: initializedCnftCollection,
    //         treeConfig: treeConfigPda,
    //         merkleTree: initializedMerkleTree,
    //         mplCoreCpiSigner: mplCoreCpiSignerKey,
    //       })
    //       .signers([playerTwo])
    //       .rpc();
    //     expect.fail("should fail with invalid player");
    //   } catch (error: any) {
    //     expect(error.error?.errorCode?.code || error.message).to.satisfy(
    //       (msg: string) =>
    //         msg.includes("unknown signer") ||
    //         msg.includes("Signature verification failed")
    //     );
    //   }
    // });
  });

  describe("Burn cNFT", () => {
    // it("burns a cnft for a player", async () => {
    //   let umi = createUmi(endpoint).use(mplBubblegum()).use(dasApi());
    //   let mplCoreCpiSignerKey = new PublicKey(
    //     "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
    //   );
    //   let configAccount = await program.account.config.fetch(configPda);
    //   let initializedCnftCollection = configAccount.cnftCollection;
    //   let initializedMerkleTree = configAccount.merkleTree;
    //   let treeConfig = findTreeConfigPda(umi, {
    //     merkleTree: publicKey(initializedMerkleTree),
    //   })[0];
    //   treeConfigPda = new PublicKey(treeConfig);
    //   const assets = await umi.rpc.getAssetsByOwner({
    //     owner: publicKey(playerOne.publicKey),
    //   });
    //   const cnft = assets.items.find(
    //     (i) =>
    //       i.compression?.compressed &&
    //       i.compression.tree == initializedMerkleTree.toString()
    //   );
    //   if (!cnft) {
    //     console.log("no cnfts found for player. assets: ", assets.items);
    //     throw new Error("no cnfts found to burn for player: ");
    //   }
    //   const assetId = cnft.id;
    //   const assetWithProof = await getAssetWithProof(umi, assetId);
    //   try {
    //     const sig = await program.methods
    //       .burnCnft(
    //         Array.from(assetWithProof.root),
    //         Array.from(assetWithProof.dataHash),
    //         Array.from(assetWithProof.creatorHash),
    //         new BN(assetWithProof.nonce),
    //         assetWithProof.index,
    //         Array.from(assetWithProof.asset_data_hash),
    //         assetWithProof.flags
    //       )
    //       .accounts({
    //         player: playerOne.publicKey,
    //         authority: wallet.publicKey,
    //         cnftCollection: initializedCnftCollection,
    //         treeConfig: treeConfigPda,
    //         merkleTree: initializedMerkleTree,
    //         mplCoreCpiSigner: mplCoreCpiSignerKey,
    //       })
    //       .remainingAccounts(
    //         assetWithProof.proof.slice(0, 5).map((i) => ({
    //           pubkey: new PublicKey(i),
    //           isWritable: false,
    //           isSigner: false,
    //         }))
    //       )
    //       .signers([playerOne])
    //       .rpc();
    //     console.log("one cnft burned");
    //     console.log("transaction signature", sig);
    //   } catch (error: any) {
    //     console.error(`something went wrong: ${error}`);
    //     if (error.logs && Array.isArray(error.logs)) {
    //       console.log("Transaction Logs:");
    //       error.logs.forEach((log: string) => console.log(log));
    //     } else {
    //       console.log("No logs available in the error.");
    //     }
    //     throw error;
    //   }
    // });
    // it("increments the number of cnfts burned for a player", async () => {
    //   let playerProgressAccount = await program.account.playerProgress.fetch(
    //     playerOneProgressPda
    //   );
    //   expect(playerProgressAccount.totalCnftsBurned).to.be.greaterThan(0);
    // });
    // it("burns a cnft for a different player", async () => {
    //   let umi = createUmi(endpoint).use(mplBubblegum()).use(dasApi());
    //   let mplCoreCpiSignerKey = new PublicKey(
    //     "CbNY3JiXdXNE9tPNEk1aRZVEkWdj2v7kfJLNQwZZgpXk"
    //   );
    //   let configAccount = await program.account.config.fetch(configPda);
    //   let initializedCnftCollection = configAccount.cnftCollection;
    //   let initializedMerkleTree = configAccount.merkleTree;
    //   let treeConfig = findTreeConfigPda(umi, {
    //     merkleTree: publicKey(initializedMerkleTree),
    //   })[0];
    //   treeConfigPda = new PublicKey(treeConfig);
    //   const assets = await umi.rpc.getAssetsByOwner({
    //     owner: publicKey(playerTwo.publicKey),
    //   });
    //   const cnft = assets.items.find(
    //     (i) =>
    //       i.compression?.compressed &&
    //       i.compression.tree == initializedMerkleTree.toString()
    //   );
    //   if (!cnft) {
    //     console.log("no cnfts found for player. assets: ", assets.items);
    //     throw new Error("no cnfts found to burn for player: ");
    //   }
    //   const assetId = cnft.id;
    //   const assetWithProof = await getAssetWithProof(umi, assetId);
    //   try {
    //     const sig = await program.methods
    //       .burnCnft(
    //         Array.from(assetWithProof.root),
    //         Array.from(assetWithProof.dataHash),
    //         Array.from(assetWithProof.creatorHash),
    //         new BN(assetWithProof.nonce),
    //         assetWithProof.index,
    //         Array.from(assetWithProof.asset_data_hash),
    //         assetWithProof.flags
    //       )
    //       .accounts({
    //         player: playerTwo.publicKey,
    //         authority: wallet.publicKey,
    //         cnftCollection: initializedCnftCollection,
    //         treeConfig: treeConfigPda,
    //         merkleTree: initializedMerkleTree,
    //         mplCoreCpiSigner: mplCoreCpiSignerKey,
    //       })
    //       .remainingAccounts(
    //         assetWithProof.proof.slice(0, 5).map((i) => ({
    //           pubkey: new PublicKey(i),
    //           isWritable: false,
    //           isSigner: false,
    //         }))
    //       )
    //       .signers([playerTwo])
    //       .rpc();
    //     console.log("one cnft burned");
    //     console.log("transaction signature", sig);
    //   } catch (error: any) {
    //     console.error(`something went wrong: ${error}`);
    //     if (error.logs && Array.isArray(error.logs)) {
    //       console.log("Transaction Logs:");
    //       error.logs.forEach((log: string) => console.log(log));
    //     } else {
    //       console.log("No logs available in the error.");
    //     }
    //     throw error;
    //   }
    // });
  });
});
