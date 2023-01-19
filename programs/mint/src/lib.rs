use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{mint_to, Mint, MintTo, Token, TokenAccount},
};

declare_id!("Hxr6EuH39zDmgPDcWTk2LpqA2ZZm6Ynhpd33nAR7v83P");

#[program]
pub mod mint {
    use super::*;

    pub fn mint(ctx: Context<MintToken>) -> Result<()> {
        let seeds = &["auth".as_bytes(), &[*ctx.bumps.get("auth").unwrap()]];
        let signer = [&seeds[..]];

        mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.auth.to_account_info(),
                    to: ctx.accounts.token_account.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                },
                &signer,
            ),
            1,
        )?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct MintToken<'info> {
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = receipient
    )]
    pub token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: mint authority PDA
    #[account(
        seeds = ["auth".as_bytes().as_ref()],
        bump,
    )]
    pub auth: UncheckedAccount<'info>,
    pub receipient: SystemAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
}
