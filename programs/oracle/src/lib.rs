use anchor_lang::{AccountsClose, prelude::*};
use anchor_lang::prelude::*;
use anchor_spl::token::*;
use anchor_spl::token::{
    Burn,
    Token,
    TokenAccount,
    Transfer,
    Mint,
    MintTo
};
use anchor_spl::associated_token::get_associated_token_address;
use solana_program::program::invoke_signed;

pub mod whirlpools;
pub mod raydium_clmm;
pub mod consts;

use whirlpools::*;
use raydium_clmm::*;
use consts::*;

declare_id!("HMrvZiD8ae5mWXDBoWjWe9ujqkfcCLNXTH1qsDfiVCiH");

#[program]
pub mod oracle {
    use super::*;

    pub fn initialize_config(
        ctx: Context<InitializeConfig>,
        num_of_pools: u8,
        protocol_list: [u8; 10],
        num_of_dependencies: [u8; 10],
    ) -> Result<()> {
        let mut config = &mut ctx.accounts.config.load_init()?;

        config.creator = ctx.accounts.creator.key();
        config.num_of_pools = num_of_pools;
        config.protocol_list = protocol_list;
        config.token_mint = ctx.accounts.token_mint.key();

        let accounts: &[AccountInfo] = ctx.remaining_accounts;
        let mut index: usize = 0;

        for i in 0..num_of_pools as usize {
            config.pool_data_list[i].pool_account = accounts[index].key();
            index += 1;
            config.pool_data_list[i].num_of_dependencies = num_of_dependencies[i];
            for j in 0..num_of_dependencies[i] as usize {
                config.pool_data_list[i].pool_dependencies[j] = accounts[index].key();
                index += 1;
            }
        }

        Ok(())
    }

    pub fn get_price(
        ctx: Context<GetPrice>
    ) -> Result<()> {
        let mut config = ctx.accounts.config.load()?;
        let accounts: &[AccountInfo] = ctx.remaining_accounts;

        let mut index: usize = 0;

        let mut price_sum: u128 = 0;

        for i in 0..config.num_of_pools as usize {
            // ORCA WHIRLPOOLS
            if config.protocol_list[i] == 0 {
                let whirlpool_price = whirlpools_spot_price(&accounts[index]);
                msg!("Price in whirlpool = {}", whirlpool_price);
                price_sum += whirlpool_price;
                index += 1;
            }
            // RAYDIUM CLMM
            if config.protocol_list[i] == 1 {
                let raydium_price = raydium_clmm_spot_price(&accounts[index]);
                msg!("Price in raydium CLMM = {}", raydium_price);
                price_sum += raydium_price;
            }
        }

        msg!("Price = {}", price_sum / config.num_of_pools as u128);

        Ok(())
    }

    pub fn create_raydium_clmm_and_whirlpool(
        ctx: Context<CreateRaydiumClmmAndWhirlpool>,
    ) -> Result<()> {
        let mut clmm_account = &mut ctx.accounts.clmm_account.load_init()?;
        let mut whirlpool_account = &mut ctx.accounts.whirlpool_account.load_init()?;
        
        clmm_account.token_mint_0 = ctx.accounts.token_mint_0.key();
        clmm_account.token_mint_1 = ctx.accounts.token_mint_1.key();

        whirlpool_account.token_mint_a = ctx.accounts.token_mint_0.key();
        whirlpool_account.token_mint_b = ctx.accounts.token_mint_1.key();

        Ok(())
    }

    pub fn simulate_price_in_clmm_and_whirlpool(
        ctx: Context<SimulatePriceInClmmAndWhirlpool>,
        price_clmm: u128,
        price_whirlpool: u128,
    ) -> Result<()> {
        let raydium_clmm = &mut ctx.accounts.raydium_clmm.load_mut()?;
        let whirlpool = &mut ctx.accounts.whirlpool.load_mut()?;
        raydium_clmm.sqrt_price_x64 = price_clmm;
        whirlpool.sqrt_price = price_whirlpool;
        Ok(())
    }

    pub fn close_account(
        ctx: Context<CloseAccount>,
    ) -> Result<()> {

        Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(mut)]
    pub creator: Signer<'info>,
    #[account(zero)]
    pub config: AccountLoader<'info, Config>,
    /// CHECK: mint of token for which oracle is created
    pub token_mint: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct GetPrice<'info> {
    pub config: AccountLoader<'info, Config>
}

#[derive(Accounts)]
pub struct CreateRaydiumClmmAndWhirlpool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(zero)]
    pub clmm_account: AccountLoader<'info, PoolState>,
    #[account(zero)]
    pub whirlpool_account: AccountLoader<'info, Whirlpool>,
    /// CHECK: mint of token_0 in pool
    pub token_mint_0: AccountInfo<'info>,
    /// CHECK: mint of token_1 in pool
    pub token_mint_1: AccountInfo<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct SimulatePriceInClmmAndWhirlpool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(mut)]
    pub raydium_clmm: AccountLoader<'info, PoolState>,
    #[account(mut)]
    pub whirlpool: AccountLoader<'info, Whirlpool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CloseAccount<'info> {
    #[account(
        mut,
        constraint = signer.key() == admin_account::ID
    )]
    pub signer: Signer<'info>,
    #[account(
        mut,
        close = signer
    )]
    pub config: AccountLoader<'info, Config>,
    pub system: Program<'info, System>,
}

#[account(zero_copy)]
pub struct Config {                     // 8 + 2005 = 2013 bytes
    pub creator: Pubkey,                // 32 bytes
    pub token_mint: Pubkey,             // 32 bytes
    pub num_of_pools: u8,               // 1 byte
    pub protocol_list: [u8; 10],        // 10 bytes
    pub pool_data_list: [PoolData; 10], // 1930 bytes
}

#[zero_copy]
pub struct PoolData {                   // 193 bytes
    pub pool_account: Pubkey,           // 32bytes
    pub num_of_dependencies: u8,        // 1 byte
    pub pool_dependencies: [Pubkey; 5], // 160 bytes
}
