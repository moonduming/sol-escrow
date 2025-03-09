use anchor_lang::prelude::*;

declare_id!("FeY4DZcAk56DpiEi7diUJi9wcwdrXkHdgBYu8ofRao8z");

#[program]
pub mod sol_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
