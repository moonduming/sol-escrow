use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, TransferChecked, transfer_checked}};

use crate::{
    constants::{ANCHOR_DISCRIMINATOR, MIN_EXPIRATION_TIME}, 
    error::ErrorCode, 
    state::{Escrow, TransactionStatus}
};


#[event]
pub struct OrderMade {
    pub maker: Pubkey,
    pub amount: u64,
    pub expiration: i64
}

#[event]
pub struct BuyerTransfers {
    pub from: Pubkey,
    pub to: Pubkey,
    pub amount: u64
}

#[event]
pub struct OrderCancelled {
    pub buyer: Pubkey,
    pub escrow: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct FundsRefunded {
    pub buyer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct OrderFunded {
    pub buyer: Pubkey,
    pub escrow: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}


#[derive(Accounts)]
pub struct CreateOrder<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(mint::token_program = token_program)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        init,
        payer = signer,
        space = ANCHOR_DISCRIMINATOR + Escrow::INIT_SPACE,
        seeds = [b"order", signer.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}


#[derive(Accounts)]
pub struct BuyerPayment<'info> {
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


#[derive(Accounts)]
pub struct OrderCancellation<'info> {
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


pub fn process_order(ctx: Context<CreateOrder>, amount: u64, expiration: i64) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow;

    let clock = Clock::get()?;
    let min_allowed_expiration = clock.unix_timestamp + MIN_EXPIRATION_TIME;

    require!(expiration >= min_allowed_expiration, ErrorCode::ExpirationTooSoon);
    require!(amount > 0, ErrorCode::AmountZero);

    escrow_account.buyer = ctx.accounts.signer.key();
    escrow_account.token_mint = ctx.accounts.mint.key();
    escrow_account.amount = amount;
    escrow_account.escrow_vault = ctx.accounts.escrow_vault.key();
    escrow_account.expiration = expiration;
    escrow_account.status = TransactionStatus::Created as u8;

    emit!(OrderMade {
        amount,
        expiration,
        maker: ctx.accounts.signer.key()
    });

    Ok(())
}


pub fn process_buyer_payment(ctx: Context<BuyerPayment>) -> Result<()> {
    let escrow_account = &mut ctx.accounts.escrow;
    let clock = Clock::get()?;

    // 状态判断
    require!(escrow_account.status == TransactionStatus::Created as u8, ErrorCode::CancellationNotAllowed);
    // 超时判断
    require!(escrow_account.expiration > clock.unix_timestamp, ErrorCode::ExpirationTooSoon);

    // 将交易金额存入托管账户
    let cpi_accounts = TransferChecked {
        from: ctx.accounts.buyer_token_account.to_account_info(),
        to: ctx.accounts.escrow_vault.to_account_info(),
        mint: ctx.accounts.mint.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info()
    };

    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(), 
        cpi_accounts
    );

    transfer_checked(cpi_ctx, escrow_account.amount, ctx.accounts.mint.decimals)?;

    emit!(BuyerTransfers {
        from: ctx.accounts.buyer.key(),
        to: ctx.accounts.escrow_vault.key(),
        amount: escrow_account.amount
    });

    emit!(OrderFunded {
        buyer: ctx.accounts.buyer.key(),
        escrow: escrow_account.key(),
        amount: escrow_account.amount,
        timestamp: clock.unix_timestamp,
    });

    escrow_account.status = TransactionStatus::Funded as u8;

    Ok(())
}


pub fn process_order_cancellation(ctx: Context<OrderCancellation>) -> Result<()> {
    let escrow_account = &ctx.accounts.escrow;
    let clock = Clock::get()?;

    // 判断是否是买家确认前的状态
    require!(escrow_account.status <= TransactionStatus::Funded as u8, ErrorCode::CancellationNotAllowed);
    // 判断订单是否超时
    require!(escrow_account.expiration > clock.unix_timestamp, ErrorCode::ExpirationTooFar);

    // 退款逻辑
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
        
        emit!(FundsRefunded {
            buyer: ctx.accounts.buyer.key(),
            amount: escrow_account.amount,
            timestamp: clock.unix_timestamp,
        });
        
        msg!("用户取消订单，退款");
    };

    let escrow_account = &mut ctx.accounts.escrow;

    escrow_account.status = TransactionStatus::Cancelled as u8;

    emit!(OrderCancelled {
        buyer: ctx.accounts.buyer.key(),
        escrow: escrow_account.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
