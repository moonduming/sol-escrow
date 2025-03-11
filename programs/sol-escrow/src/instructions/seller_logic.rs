use anchor_lang::prelude::*;

use crate::{error::ErrorCode, state::{Escrow, TransactionStatus}};


#[derive(Accounts)]
pub struct SellerConfirmation<'info> {
    pub seller: SystemAccount<'info>,
    pub buyer: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"order", buyer.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,


    pub system_program: Program<'info, System>,
}


pub fn process_seller_confirmation(ctx: Context<SellerConfirmation>) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow;
    let clock = Clock::get()?;

    require!(escrow_account.expiration > clock.unix_timestamp, ErrorCode::ExpirationTooFar);
    require!(escrow_account.status == TransactionStatus::Funded as u8, ErrorCode:: SellerConfirmationNotAllowed);

    escrow_account.seller = Some(ctx.accounts.seller.key());
    escrow_account.status = TransactionStatus::InTransit as u8;
    
    Ok(())
}
