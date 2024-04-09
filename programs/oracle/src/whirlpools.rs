use crate::*;
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

pub fn whirlpools_spot_price(
    whirlpool_account_info: &AccountInfo,
) -> u128 {
    let whirlpool = get_whirlpool_from_account_info(whirlpool_account_info);

    let mut sqrt_price: u128 = whirlpool.sqrt_price;
    return sqrt_price / u128::pow(2, 64) * sqrt_price / u128::pow(2, 64);
}

fn get_whirlpool_from_account_info(account_info: &AccountInfo) -> Whirlpool {
    let data: &[u8] = &account_info.try_borrow_data().expect("");
    let whirlpool: Whirlpool = Whirlpool::try_deserialize(&mut data.try_into().expect("R")).expect("REASON");
    return whirlpool
}


#[account(zero_copy(unsafe))]
#[repr(packed)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Whirlpool {
    pub whirlpools_config: Pubkey, // 32
    pub whirlpool_bump: [u8; 1],   // 1

    pub tick_spacing: u16,          // 2
    pub tick_spacing_seed: [u8; 2], // 2

    // Stored as hundredths of a basis point
    // u16::MAX corresponds to ~6.5%
    pub fee_rate: u16, // 2

    // Portion of fee rate taken stored as basis points
    pub protocol_fee_rate: u16, // 2

    // Maximum amount that can be held by Solana account
    pub liquidity: u128, // 16

    // MAX/MIN at Q32.64, but using Q64.64 for rounder bytes
    // Q64.64
    pub sqrt_price: u128,        // 16
    pub tick_current_index: i32, // 4

    pub protocol_fee_owed_a: u64, // 8
    pub protocol_fee_owed_b: u64, // 8

    pub token_mint_a: Pubkey,  // 32
    pub token_vault_a: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_a: u128, // 16

    pub token_mint_b: Pubkey,  // 32
    pub token_vault_b: Pubkey, // 32

    // Q64.64
    pub fee_growth_global_b: u128, // 16

    pub reward_last_updated_timestamp: u64, // 8

    pub reward_infos: [WhirlpoolRewardInfo; NUM_REWARDS], // 384
}

// Number of rewards supported by Whirlpools
pub const NUM_REWARDS: usize = 3;

#[zero_copy(unsafe)]
#[repr(packed)]
#[derive(Default, Debug, PartialEq, Eq)]
// #[derive(Copy, AnchorSerialize, AnchorDeserialize, Default, Debug, PartialEq)]
pub struct WhirlpoolRewardInfo {
    /// Reward token mint.
    pub mint: Pubkey,
    /// Reward vault token account.
    pub vault: Pubkey,
    /// Authority account that has permission to initialize the reward and set emissions.
    pub authority: Pubkey,
    /// Q64.64 number that indicates how many tokens per second are earned per unit of liquidity.
    pub emissions_per_second_x64: u128,
    /// Q64.64 number that tracks the total tokens earned per unit of liquidity since the reward
    /// emissions were turned on.
    pub growth_global_x64: u128,
}

impl Whirlpool {
    pub const LEN: usize = 8 + 261 + 384;
}