use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{error::ErrorCode, state::{Escrow, TransactionStatus}};


#[derive(Accounts)]
pub struct EscrowRelease<'info> {
    pub buyer: SystemAccount<'info>,
    pub seller: SystemAccount<'info>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"order", buyer.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = seller,
        associated_token::token_program = token_program
    )]
    pub seller_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}


#[derive(Accounts)]
pub struct TimeoutCheck<'info> {
    pub buyer: SystemAccount<'info>,
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        seeds = [b"order", buyer.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_vault: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = buyer,
        associated_token::token_program = token_program
    )]
    pub buyer_token_account: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}


pub fn process_escrow_release(ctx: Context<EscrowRelease>) -> Result<()> {
    let escrow_account = &ctx.accounts.escrow;

    require!(escrow_account.status == TransactionStatus::InTransit as u8, ErrorCode::FundsReleaseNotAllowed);

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"order",
        ctx.accounts.buyer.to_account_info().key.as_ref(),
        &[ctx.bumps.escrow]
    ]];

    let cpi_accounts = TransferChecked {
        from: ctx.accounts.escrow_vault.to_account_info(),
        to: ctx.accounts.seller_token_account.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.escrow.to_account_info()
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(), 
        cpi_accounts, 
        signer_seeds
    );

    transfer_checked(cpi_ctx, escrow_account.amount, ctx.accounts.mint.decimals)?;

    msg!("托管账户释放资金完毕");

    let escrow_account = &mut ctx.accounts.escrow;
    escrow_account.status = TransactionStatus::Success as u8;

    Ok(())
}


pub fn process_timeout(ctx: Context<TimeoutCheck>) -> Result<()> {
    let escrow_account = &ctx.accounts.escrow;
    let clock = Clock::get()?;

    // 订单超出处理
    if escrow_account.expiration <= clock.unix_timestamp {
        if escrow_account.status == TransactionStatus::Funded as u8 {
            let signer_seeds: &[&[&[u8]]] = &[&[
                b"order",
                ctx.accounts.buyer.to_account_info().key.as_ref(),
                &[ctx.bumps.escrow]
            ]];
    
            let cpi_accounts = TransferChecked {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.buyer_token_account.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info()
            };
    
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(), 
                cpi_accounts, 
                signer_seeds
            );
    
            transfer_checked(cpi_ctx, escrow_account.amount, ctx.accounts.mint.decimals)?;

            msg!("订单超时退换资金给买家");
        };

        let escrow_account = &mut ctx.accounts.escrow;
        escrow_account.status = TransactionStatus::Expired as u8;
    }

    Ok(())
}
