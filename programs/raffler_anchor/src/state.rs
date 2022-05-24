use {
    anchor_lang::*,
    anchor_lang::prelude::*,
    anchor_spl::token::{Token, TokenAccount, Mint},
    anchor_spl::associated_token::{AssociatedToken}
};

pub const MOON_KEY: &str = "MoonJpLwzBSu2SEeXe42rDySA21NifCuPBDPr5jExET";

pub const RAFFLE_ENTRY_OFFSET: usize = 8 + 32 + 4;
pub const RAFFLE_ENTRY_SIZE: usize = 33;


#[derive(Accounts)]
pub struct InitTokenAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint_cost: Account<'info, Mint>,
    pub mint_prize: Account<'info, Mint>,
    #[account(mut)]
    /// CHECK: yeah
    pub token_prize: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: yeah
    pub token_cost: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: yeah
    pub escrow_token_prize: UncheckedAccount<'info>,
    #[account(mut)]
    /// CHECK: yeah
    pub escrow_token_cost: UncheckedAccount<'info>,
    pub associated_token: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: yeah
    pub raffle: UncheckedAccount<'info>,
    /// CHECK: yeah
    pub recipient: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct CreateRaffle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint_cost: Account<'info, Mint>,
    #[account(
        mut,
        constraint = payer.key == &token_prize.owner,
        constraint = mint_prize.key() == token_prize.mint
    )]
    pub token_prize: Account<'info, TokenAccount>,
    pub mint_prize: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        space = 1000,
        seeds = [payer.key().as_ref(), mint_cost.key().as_ref(), mint_prize.key().as_ref()], bump,
    )]
    pub raffle: Box<Account<'info, RaffleAccount>>,
    #[account(
        zero
    )]
    pub fixed_raffle: Box<Account<'info, FixedTicketAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(
        mut,
        constraint = raffle.key() == escrow_token_prize.owner,
        constraint = escrow_token_prize.mint == mint_prize.key()
    )]
    pub escrow_token_prize: Account<'info, TokenAccount>
}

#[derive(Accounts)]
pub struct CloseRaffle<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub mint_cost: Account<'info, Mint>,
    #[account(
        mut,
        constraint = payer.key == &token_prize.owner || payer.key.to_string() == MOON_KEY,
        constraint = mint_prize.key() == token_prize.mint
    )]
    pub token_prize: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = payer.key == &token_cost.owner || payer.key.to_string() == MOON_KEY,
        constraint = mint_cost.key() == token_cost.mint
    )]
    pub token_cost: Box<Account<'info, TokenAccount>>,
    pub mint_prize: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = raffle.owner == *payer.key || payer.key.to_string() == MOON_KEY,
        constraint = raffle.mint == mint_cost.key(),
        constraint = raffle.prize == mint_prize.key()
    )]
    pub raffle: Box<Account<'info, RaffleAccount>>,
    #[account(
        mut,
//        constraint = fixed_raffle.raffle_id == raffle.key()
    )]
    /// CHECK: see constraint
    pub fixed_raffle: UncheckedAccount<'info>, // FixedTicketAccount
    pub system_program: Program<'info, System>,
    pub token_program:  Program<'info, Token>,
    #[account(
        mut,
        constraint = raffle.key() == escrow_token_prize.owner,
        constraint = escrow_token_prize.mint == mint_prize.key()
    )]
    pub escrow_token_prize: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = raffle.key() == escrow_token_cost.owner,
        constraint = escrow_token_cost.mint == mint_cost.key()
    )]
    pub escrow_token_cost: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = moon.key().to_string() == MOON_KEY,
    )]
    pub moon: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct BuyTicket<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub mint_cost: Account<'info, Mint>,
    #[account(
        mut,
        constraint = payer.key == &token_cost.owner,
        constraint = mint_cost.key() == token_cost.mint
    )]
    pub token_cost: Account<'info, TokenAccount>,
    pub mint_prize: Account<'info, Mint>,
    #[account(
        mut,
        constraint = raffle.mint == mint_cost.key(),
        constraint = raffle.prize == mint_prize.key()
    )]
    pub raffle: Box<Account<'info, RaffleAccount>>,
    #[account(
        mut,
//        constraint = fixed_raffle.raffle_id == raffle.key()
    )]
    /// CHECK: see constraint
    pub fixed_raffle: UncheckedAccount<'info>, // FixedTicketAccount
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    #[account(
        mut,
        constraint = raffle.key() == escrow_token_cost.owner,
        constraint = escrow_token_cost.mint == mint_cost.key()
    )]
    pub escrow_token_cost: Account<'info, TokenAccount>

}

#[derive(Accounts)]
pub struct DrawWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    pub recipient: SystemAccount<'info>,
    pub mint_cost: Account<'info, Mint>,
    pub mint_prize: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = raffle.owner == *payer.key,
        constraint = raffle.mint == mint_cost.key(),
        constraint = raffle.prize == mint_prize.key()
    )]
    pub raffle: Box<Account<'info, RaffleAccount>>,
    #[account(
        mut,
        constraint = recipient.key == &token_prize.owner,
        constraint = mint_prize.key() == token_prize.mint
    )]
    pub token_prize: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = raffle.key() == escrow_token_prize.owner,
        constraint = escrow_token_prize.mint == mint_prize.key()
    )]
    pub escrow_token_prize: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    /// CHECK: see constraint
    pub fixed_raffle: UncheckedAccount<'info>, // FixedTicketAccount
}

#[derive(Accounts)]
pub struct SetWinner<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub mint_cost: Account<'info, Mint>,
    pub mint_prize: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = raffle.owner == *payer.key || payer.key.to_string() == MOON_KEY,
        constraint = raffle.mint == mint_cost.key(),
        constraint = raffle.prize == mint_prize.key()
    )]
    pub raffle: Box<Account<'info, RaffleAccount>>,
    #[account(
        mut,
//        constraint = fixed_raffle.raffle_id == raffle.key()
    )]
    /// CHECK: see constraint
    pub fixed_raffle: UncheckedAccount<'info>, // FixedTicketAccount
    /// CHECK: RecentSlothash
    pub slot_hashes: UncheckedAccount<'info>
}

// PDA of < owner - token_mint - prize_mint >
#[account]
pub struct RaffleAccount {
    pub id: Pubkey,
    pub owner: Pubkey,
    pub mint: Pubkey,
    pub prize: Pubkey,
    pub prize_quantity: u64,
    pub tickets_purchased: u64,
    pub price: u64,
    pub start: i64,
    pub date_created: i64,
    pub end: i64,
    pub ticket_count: u64,
    pub max_entries: u64,
    pub per_win: u64,
    pub cost_decimals: u8,
    pub prize_decimals: u8,
    pub win_multiple: bool,
    pub bump: u8,
    pub burn: bool,
    pub fixed: bool,
    pub unique_entries: u16,
    pub winners_selected: bool,
    pub sent_out: u8,
    pub description: String,
    pub nft_image: String,
    pub nft_uri: String,
    pub winners: Vec<u64>
}

#[account]
pub struct FixedTicketAccount {
    pub raffle_id: Pubkey,
    pub entries: Vec<FixedEntry>,
}

//
//
//

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct FixedEntry {
    pub buyer: Pubkey,
    pub wins: u8
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct CreateRaffleData {
    pub prize_quantity: u64,
    pub price:    u64,
    pub start:    i64,
    pub end:      i64,
    pub max_entries: u64,
    pub per_win:     u64,
    pub cost_decimals: u8,
    pub prize_decimals: u8,
    pub win_multiple: bool,
    pub burn: bool,
    pub fixed: bool,
    pub description: String,
    pub nft_uri: String,
    pub nft_image: String
}
