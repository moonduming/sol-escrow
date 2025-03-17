use anchor_lang::prelude::*;
use anchor_spl::token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked};

use crate::{error::ErrorCode, state::{Escrow, TransactionStatus}};

#[event]
pub struct SellerConfirmed {
    pub escrow: Pubkey,
    pub seller: Pubkey,
    pub buyer: Pubkey,
    pub timestamp: i64,
}

#[derive(Accounts)]
pub struct SellerConfirmation<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
    pub buyer: SystemAccount<'info>,
    pub nft_mint: Option<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [b"order", buyer.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(mut)]
    pub seller_nft_account: Option<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub buyer_nft_account: Option<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>
}


pub fn process_seller_confirmation(ctx: Context<SellerConfirmation>) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow;
    let clock = Clock::get()?;

    require!(escrow_account.expiration > clock.unix_timestamp, ErrorCode::ExpirationTooFar);
    require!(escrow_account.status == TransactionStatus::Funded as u8, ErrorCode::SellerConfirmationNotAllowed);

    if escrow_account.is_nft {
        // 获取seller_nft_account，没有报错
        let seller_nft_account = match &ctx.accounts.seller_nft_account {
            Some(nft_account) => nft_account,
            None => return Err(ErrorCode::MissingNftAccount.into()),
        };

        let nft_mint = escrow_account.nft_mint.unwrap();
        require!(seller_nft_account.mint == nft_mint, ErrorCode::InvalidNftAccount);
        
        // 验证卖家是否拥有此nft，nft是否有效
        require!(seller_nft_account.owner == ctx.accounts.seller.key(), ErrorCode::InvalidNftOwner);
        require!(seller_nft_account.amount == 1, ErrorCode::InvalidNftAmount);

        // 将nft所有权转交给买家
        let nft_mint = match &ctx.accounts.nft_mint {
            Some(nft_mint) => nft_mint,
            None => return Err(ErrorCode::MissingNftMint.into())
        };

        let buyer_nft_account = match &ctx.accounts.buyer_nft_account {
            Some(buyer_nft_account) => buyer_nft_account,
            None => return Err(ErrorCode::MissingBuyerNftAccount.into())
        };

        require!(buyer_nft_account.owner == ctx.accounts.buyer.key(), ErrorCode::InvalidNftOwner);

        let cpi_accounts = TransferChecked {
            from: seller_nft_account.to_account_info(),
            to: buyer_nft_account.to_account_info(),
            mint: nft_mint.to_account_info(),
            authority: ctx.accounts.seller.to_account_info()
        };

        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(), 
            cpi_accounts
        );

        transfer_checked(cpi_ctx, 1, 0)?;
    }

    escrow_account.seller = Some(ctx.accounts.seller.key());
    escrow_account.status = TransactionStatus::InTransit as u8;

    emit!(SellerConfirmed {
        escrow: escrow_account.key(),
        seller: ctx.accounts.seller.key(),
        buyer: ctx.accounts.buyer.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
