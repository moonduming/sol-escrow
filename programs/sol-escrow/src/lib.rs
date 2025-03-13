use anchor_lang::prelude::*;

pub mod constants;
pub mod state;
pub mod instructions;
pub mod error;

use instructions::*;

declare_id!("FeY4DZcAk56DpiEi7diUJi9wcwdrXkHdgBYu8ofRao8z");

#[program]
pub mod sol_escrow {
    use super::*;

    // 创建订单
    pub fn create_order(
        ctx: Context<CreateOrder>, 
        amount: u64, 
        expiration: i64, 
        nft_mint: Option<Pubkey>,
        collection_mint: Option<Pubkey>,
        buyer_nft_account: Option<Pubkey>,
        is_nft: bool
    ) -> Result<()> {
        msg!("创建订单");
        process_order(ctx, amount, expiration, nft_mint, collection_mint, buyer_nft_account, is_nft)
    }

    // 买家付款确认
    pub fn buyer_payment(ctx: Context<BuyerPayment>) -> Result<()> {
        msg!("买家确认付款");
        process_buyer_payment(ctx)
    }

    // 买家取消订单
    pub fn order_cancellation(ctx: Context<OrderCancellation>) -> Result<()> {
        msg!("买家取消订单");
        process_order_cancellation(ctx)
    }

    // 卖家确认
    pub fn seller_confirmation(ctx: Context<SellerConfirmation>) -> Result<()> {
        msg!("卖家确认");
        process_seller_confirmation(ctx)
    }

    // 合约转账
    pub fn escrow_release(ctx: Context<EscrowRelease>) -> Result<()> {
        msg!("托管账户资金释放");
        process_escrow_release(ctx)
    }

    // 超时处理
    pub fn timeou_check(ctx: Context<TimeoutCheck>) -> Result<()> {
        msg!("超时判断");
        process_timeout(ctx)
    }
}
