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
} from "@solana/spl-token";
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
  });

  it("Is create order", async () => {
    const now = Math.floor(Date.now() / 1000);

    await program.methods.createOrder(
      new anchor.BN(1000),
      new anchor.BN(now + 3600)
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
    let seller = Keypair.generate();
    const sellerToken = await getOrCreateAssociatedTokenAccount(
      connection,
      payer,
      mint,
      seller.publicKey
    );

    // 卖家确认指令对象
    const sellerConfirmationIx = await program.methods.sellerConfirmation()
      .accounts({
        seller: seller.publicKey,
        buyer: payer.publicKey,
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

    await provider.sendAndConfirm(tx);

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
