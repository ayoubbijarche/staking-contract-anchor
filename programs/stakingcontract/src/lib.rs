use anchor_lang::prelude::*;
use anchor_spl::token::{self , Token,  MintTo, Transfer};

declare_id!("8xHxL2EX8StDf1VQbNQRt4D7UvNPtBfn8nnPND7ZvVzd");

#[program]
mod stakingcontract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, start_slot: u64, end_slot: u64) -> Result<()> {
        let poolinfo = &mut ctx.accounts.pool_info;
        poolinfo.admin = ctx.accounts.admin.key();
        poolinfo.start_slot = start_slot;
        poolinfo.end_slot = end_slot;
        poolinfo.token = ctx.accounts.staking_token.key();
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let userinfo = &mut ctx.accounts.user_info;
        let clock = Clock::get()?;
        
        if userinfo.amount > 0 {
            let reward = (clock.slot - userinfo.deposit_slot) - userinfo.debt_reward;
            let cpi_accounts = MintTo {
                mint: ctx.accounts.staking_token.to_account_info(),
                to: ctx.accounts.user_staking_wallet.to_account_info(),
                authority: ctx.accounts.admin_staking_wallet.to_account_info(),
            };

            let cpi_program = ctx.accounts.token_program.to_account_info();
            let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
            token::mint_to(cpi_ctx, reward)?;
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_staking_wallet.to_account_info(),
            to: ctx.accounts.admin_staking_wallet.to_account_info(),
            authority: ctx.accounts.user.to_account_info()
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        userinfo.amount += amount;
        userinfo.deposit_slot = clock.slot;
        userinfo.debt_reward = 0;
        Ok(())
    }

    pub fn unstake(ctx: Context<Unstake>) -> Result<()> {
        let userinfo = &mut ctx.accounts.user_info;
        let clock = Clock::get()?;
        let reward = (clock.slot - userinfo.deposit_slot) - userinfo.debt_reward;

        let cpi_accounts = MintTo {
            mint: ctx.accounts.staking_token.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.admin_staking_wallet.to_account_info()
        };

        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::mint_to(cpi_ctx, reward)?;
        
        let cpi_accounts = Transfer {
            from: ctx.accounts.admin_staking_wallet.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.admin.to_account_info(),
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, userinfo.amount)?;

        userinfo.amount = 0;
        userinfo.deposit_slot = 0;
        userinfo.debt_reward = 0;

        Ok(())
    }

    pub fn claim_reward(ctx: Context<Claimreward>) -> Result<()> {
        let user_info = &mut ctx.accounts.user_info;
        let clock = Clock::get()?;
        let reward = (clock.slot - user_info.deposit_slot) - user_info.debt_reward;
        let cpi_accounts = MintTo {
            mint: ctx.accounts.staking_token.to_account_info(),
            to: ctx.accounts.user_staking_wallet.to_account_info(),
            authority: ctx.accounts.admin.to_account_info()
        };
        
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::mint_to(cpi_ctx, reward)?;

        user_info.debt_reward += reward;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        space = 8 + PoolInfo::LEN
    )]
    pub pool_info: Account<'info, PoolInfo>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub staking_token: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user: Signer<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: UncheckedAccount<'info>,
    #[account(
        init,
        payer = user,
        space = 8 + UserInfo::LEN
    )]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_staking_wallet: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin_staking_wallet: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub staking_token: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct Unstake<'info> {
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_staking_wallet: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin_staking_wallet: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub staking_token: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program : Program<'info , System>
}

#[derive(Accounts)]
pub struct Claimreward<'info> {
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: UncheckedAccount<'info>,
    #[account(mut)]
    pub user_info: Account<'info, UserInfo>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub user_staking_wallet: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin_staking_wallet: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub staking_token: UncheckedAccount<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct PoolInfo {
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub admin: Pubkey,
    pub start_slot: u64,
    pub end_slot: u64,
    pub token: Pubkey
}

#[account]
pub struct UserInfo {
    pub amount: u64,
    pub debt_reward: u64,
    pub deposit_slot: u64
}

impl PoolInfo {
    pub const LEN: usize = 32 + 8 + 8 + 32;
}

impl UserInfo {
    pub const LEN: usize = 8 + 8 + 8;
}
