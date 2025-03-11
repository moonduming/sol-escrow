import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolEscrow } from "../target/types/sol_escrow";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getOrCreateAssociatedTokenAccount,
} from "@solana/spl-token";

describe("sol-escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env()
  anchor.setProvider(provider);

  const program = anchor.workspace.SolEscrow as Program<SolEscrow>;

  const connection = provider.connection;
  const payer = (provider.wallet as anchor.Wallet).payer;

  let mint: PublicKey;

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
    const tokenAccount = await getOrCreateAssociatedTokenAccount(
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
  });

  it("order cancellation", async () => {
    await program.methods.orderCancellation().accounts({
      buyer: payer.publicKey,
      mint,
      tokenProgram: TOKEN_PROGRAM_ID
    }).rpc();

    const [escrowAddress] = PublicKey.findProgramAddressSync(
      [Buffer.from("order"), payer.publicKey.toBuffer()],
      program.programId
    );

    const escrowData = await program.account.escrow.fetch(escrowAddress);
    console.log("order cancellation: ", escrowData);
  });

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
  });
});
