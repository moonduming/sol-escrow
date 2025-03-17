import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolEscrow } from "../target/types/sol_escrow";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
  Account,
  transfer
} from "@solana/spl-token";
import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import { generateSigner, percentAmount, KeypairSigner, some, keypairIdentity } from "@metaplex-foundation/umi";
import { createNft, mplTokenMetadata, verifyCollectionV1, findMetadataPda } from "@metaplex-foundation/mpl-token-metadata";
import assert from "assert";


describe("sol-escrow", () => {
  let Created = 0;
  let Funded = 1;
  let Cancelled = 2;
  let Success = 3;
  let Expired = 4;
  
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.SolEscrow as Program<SolEscrow>;

  const connection = provider.connection;
  const payer = (provider.wallet as anchor.Wallet).payer;

  let mint: PublicKey;
  let tokenAccount: Account;

  // 创建nft所需变量
  let collectionMint: KeypairSigner;
  let nft1: KeypairSigner;
  let nft2: KeypairSigner;
  let seller: Keypair;
  let sellerNftAccount1: Account;
  let sellerNftAccount2: Account;

  
  const umi = createUmi("http://127.0.0.1:8899").use(mplTokenMetadata());
  const umiSigner = umi.eddsa.createKeypairFromSecretKey(payer.secretKey);

  // 设置 Umi 的身份为 umiSigner
  umi.use(keypairIdentity(umiSigner));

  async function getTokenBalance(tokenAccount: PublicKey): Promise<number> {
    const balanceInfo = await connection.getTokenAccountBalance(tokenAccount);
    return balanceInfo.value.uiAmount!;
  }

  before(async () => {

    // 创建新的SPL代币
    mint = await createMint(
      connection,
      payer,
      payer.publicKey,
      null,
      2,
    );
    console.log("Mint created:", mint.toBase58())

    // 买家创建关联代币账户(ATA)
    tokenAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      payer.publicKey
    );

    // 给代币账户铸造一些代币
    await mintTo(
      connection,
      payer,
      mint,
      tokenAccount.address,
      payer,
      1000 * 100
    );

    // 🏗️ **Step 1: 创建集合 NFT**
    console.log("🚀 创建 NFT 集合...");
    collectionMint = generateSigner(umi);
    await createNft(umi, {
      mint: collectionMint,
      name: "Test NFT Collection",
      uri: "https://example.com/collection.json",
      sellerFeeBasisPoints: percentAmount(0),
      isCollection: true
    }).sendAndConfirm(umi);
    console.log("✅ NFT 集合创建成功:", collectionMint.publicKey);

    // 🏗️ **Step 2: 创建 NFT 1**
    console.log("🚀 创建 NFT 1...");
    nft1 = generateSigner(umi);
    await createNft(umi, {
      mint: nft1,
      name: "test NFT 1",
      uri: "https://example.com/nft1.json",
      sellerFeeBasisPoints: percentAmount(0),
      collection: some({
        key: collectionMint.publicKey,
        verified: false
      })
    }).sendAndConfirm(umi);
    console.log("✅ NFT 1 创建成功:", nft1.publicKey);

    // 🏗️ **Step 3: 创建 NFT 2**
    console.log("🚀 创建 NFT 2...");
    nft2 = generateSigner(umi);
    await createNft(umi, {
      mint: nft2,
      name: "test NFT 2",
      uri: "https://example.com/nft2.json",
      sellerFeeBasisPoints: percentAmount(0),
      collection: some({
        key: collectionMint.publicKey,
        verified: false
      })
    }).sendAndConfirm(umi);
    console.log("✅ NFT 2 创建成功:", nft2.publicKey);

    // **Step 3: 验证 NFT 是否属于集合**
    const metadata = findMetadataPda(umi, { mint: nft1.publicKey });
    const metadata2 = findMetadataPda(umi, { mint: nft2.publicKey });
    await verifyCollectionV1(umi, {
      metadata,
      collectionMint: collectionMint.publicKey,
      authority: umi.identity,
    }).sendAndConfirm(umi);

    await verifyCollectionV1(umi, {
      metadata: metadata2,
      collectionMint: collectionMint.publicKey,
      authority: umi.identity,
    }).sendAndConfirm(umi);
    console.log("✅ NFT 已被集合验证！");

    // 🏗️ **Step 4: 绑定 NFT 到卖家**
    seller = Keypair.generate();
    console.log("🚀 生成卖家 Keypair:", seller.publicKey.toBase58());

    // 创建卖家的 NFT 关联账户 (ATA)
    sellerNftAccount1 = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      new PublicKey(nft1.publicKey),
      seller.publicKey
    );

    sellerNftAccount2 = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      new PublicKey(nft2.publicKey),
      seller.publicKey
    );

    console.log("✅ 卖家 NFT 账户 1:", sellerNftAccount1.address.toBase58());
    console.log("✅ 卖家 NFT 账户 2:", sellerNftAccount2.address.toBase58());

    // **转移 NFT 给卖家**
    console.log("🚀 发送 NFT 1 给卖家...");
    const payerNftAccount = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      new PublicKey(nft1.publicKey),
      payer.publicKey
    );
    await transfer(
      connection,
      payer,
      payerNftAccount.address,
      sellerNftAccount1.address,
      payer.publicKey,
      1
    );

    console.log("🚀 发送 NFT 2 给卖家...");
    const payerNftAccount2 = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      new PublicKey(nft2.publicKey),
      payer.publicKey
    );
    await transfer(
      connection,
      payer,
      payerNftAccount2.address,
      sellerNftAccount2.address,
      payer.publicKey,
      1
    );

    console.log("✅ NFT 已转移到卖家账户");
  });

  it("Is create order", async () => {
    console.log("🚀 为买家创建nft账户")
    const buyerNftAccount = await getOrCreateAssociatedTokenAccount(
      connection,                   // 与 Solana 节点通信的连接
      payer,                        // 支付费用的账户（可以是创建者）
      new PublicKey(nft1.publicKey),  // NFT 的 mint 地址（转换为 web3.js 的 PublicKey）
      payer.publicKey               // 买家的钱包地址
    );
    console.log("✅ NFT 账户创建成功")

    const now = Math.floor(Date.now() / 1000);

    await program.methods.createOrder(
      new anchor.BN(1000),
      new anchor.BN(now + 3600),
      new PublicKey(nft1.publicKey),
      buyerNftAccount.address,
      true
    ).accounts({
      mint,
      tokenProgram: TOKEN_PROGRAM_ID
    }).rpc();
    
    const [escrowAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), payer.publicKey.toBuffer()],
      program.programId
    );

    const escrowData = await program.account.escrow.fetch(escrowAddress);
    console.log("escrowData: ", escrowData);

    assert.strictEqual(escrowData.amount.toNumber(), 1000, "订单金额不正确");
    assert(escrowData.expiration.toNumber() > now, "订单过期时间不合理");
  });

  it("Buyer Payment",async () => {
    await program.methods.buyerPayment().accounts({
      buyer: payer.publicKey,
      mint,
      tokenProgram: TOKEN_PROGRAM_ID
    }).rpc();

    const [escrowAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), payer.publicKey.toBuffer()],
      program.programId
    );

    const escrowData = await program.account.escrow.fetch(escrowAddress);
    console.log("Buyer Payment: ", escrowData);

    assert.strictEqual(escrowData.status, Funded, "订单状态未更新为 Funded");
    // 校验托管账户余额
    const escrowVaultBalance = await getTokenBalance(escrowData.escrowVault);
    assert.strictEqual(escrowVaultBalance, 1000 / 100, "托管账户余额不正确");
  });

  // it("order cancellation", async () => {
  //   await program.methods.orderCancellation().accounts({
  //     buyer: payer.publicKey,
  //     mint,
  //     tokenProgram: TOKEN_PROGRAM_ID
  //   }).rpc();

  //   const [escrowAddress] = PublicKey.findProgramAddressSync(
  //     [Buffer.from("order"), payer.publicKey.toBuffer()],
  //     program.programId
  //   );

  //   const escrowData = await program.account.escrow.fetch(escrowAddress);
  //   console.log("order cancellation: ", escrowData);

  //   // 添加校验：订单状态应为 Cancelled
  //   assert.strictEqual(escrowData.status, Expired, "订单状态未更新为 Expired");
  //   // 买家退款到账，假设 buyerTokenAccount 为买家的 ATA
  //   const buyerBalance = await getTokenBalance(tokenAccount.address);
  //   assert.strictEqual(buyerBalance, 1000, "买家未收到退款");
  //   // 托管账户余额应归零
  //   const escrowVaultBalance = await getTokenBalance(escrowData.escrowVault);
  //   assert.strictEqual(escrowVaultBalance, 0, "托管账户余额未归零");
  // });

  it("seller confirmation", async () => {
    // let seller = Keypair.generate();
    const sellerToken = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      seller.publicKey
    );

    // 获取订单信息
    const [escrowAddress1] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), payer.publicKey.toBuffer()],
      program.programId
    );

    const escrowData1 = await program.account.escrow.fetch(escrowAddress1);

    // 卖家确认指令对象
    const sellerConfirmationIx = await program.methods.sellerConfirmation()
      .accounts({
        seller: seller.publicKey,
        buyer: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        sellerNftAccount: sellerNftAccount1.address,
        buyerNftAccount: escrowData1.buyerNftAccount,
        nftMint: nft1.publicKey,
      }).instruction();

    // 合约转账指令对象
    const escrowReleaseIx = await program.methods.escrowRelease()
      .accounts({
        buyer: payer.publicKey,
        seller: seller.publicKey,
        mint,
        tokenProgram: TOKEN_PROGRAM_ID
      }).instruction()

    // 创建新交易
    const tx = new anchor.web3.Transaction();
    tx.add(sellerConfirmationIx, escrowReleaseIx);

    await provider.sendAndConfirm(tx, [seller]);

    const [escrowAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), payer.publicKey.toBuffer()],
      program.programId
    );

    const escrowData = await program.account.escrow.fetch(escrowAddress);
    console.log("seller confirmation: ", escrowData);

    // 添加校验：订单状态应为 Success
    assert.strictEqual(escrowData.status, Success, "订单状态未更新为 Success");
    // 校验卖家到账：卖家账户余额应为10
    const sellerBalance = await getTokenBalance(sellerToken.address);
    assert.strictEqual(sellerBalance, 10, "卖家未收到正确资金");
    // 托管账户余额应为0
    const escrowVaultBalance = await getTokenBalance(escrowData.escrowVault);
    assert.strictEqual(escrowVaultBalance, 0, "托管账户余额未归零");
  });
});
