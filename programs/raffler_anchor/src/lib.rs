pub mod error;
pub mod state;
pub mod security;

use {
    anchor_lang::prelude::*,
    crate::{error::*, state::*},
    std::collections::BTreeSet,
};

declare_id!("RafXcAJfB3wVzyH7cHuDqyVjYeQ4Qy8RjQEbCPvttK9");

#[program]
pub mod raffler_anchor {
    use super::*;

    pub fn create_raffle(ctx: Context<CreateRaffle>, data: CreateRaffleData) -> Result<()> {
        if data.start >= data.end || data.prize_quantity == 0 || data.price == 0 || data.per_win == 0 || ctx.accounts.token_prize.amount < data.prize_quantity {
            return err!(CustomError::InputError);
        }

        // this needs to be divisible
        if data.prize_quantity % data.per_win != 0 {
            return err!(CustomError::DivisibleError);
        }

        // later
        if data.fixed == false {
            return err!(CustomError::FixedError);
        }

        let clock = Clock::get()?;

        // raffles can't be longer than two weeks unless they're set to be open forever, in which case they must sell out
        if data.end > clock.unix_timestamp + 60 * 60 * 24 * 14 && data.end != i64::MAX {
            return err!(CustomError::TimeError);
        }

        let raffle = &mut ctx.accounts.raffle;
        raffle.id = raffle.key();
        raffle.owner = *ctx.accounts.payer.key;
        raffle.mint = ctx.accounts.mint_cost.key();
        raffle.prize = ctx.accounts.mint_prize.key();
        raffle.prize_quantity = data.prize_quantity;
        raffle.tickets_purchased = 0;
        raffle.cost_decimals = data.cost_decimals;
        raffle.prize_decimals = data.prize_decimals;
        raffle.price = data.price;
        raffle.start = data.start;
        raffle.date_created = clock.unix_timestamp;
        raffle.end = data.end;
        raffle.ticket_count = 0;
        raffle.max_entries = data.max_entries;
        raffle.per_win = data.per_win;
        raffle.win_multiple = data.win_multiple;
        raffle.description = data.description;
        raffle.bump = *ctx.bumps.get("raffle").unwrap();
        raffle.burn = data.burn;
        raffle.nft_image = data.nft_image;
        raffle.nft_uri = data.nft_uri;
        raffle.fixed = data.fixed;

        if data.cost_decimals != ctx.accounts.mint_cost.decimals || data.prize_decimals != ctx.accounts.mint_prize.decimals {
            return err!(CustomError::DecimalError);
        }

        let prize_decimals = (ctx.accounts.mint_prize.decimals - raffle.prize_decimals) as u32;

        anchor_spl::token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                from: ctx.accounts.token_prize.to_account_info(),
                to:  ctx.accounts.escrow_token_prize.to_account_info(),
                authority:  ctx.accounts.payer.to_account_info()
            }),
            raffle.prize_quantity * 10_u64.pow(prize_decimals),
        )?;

        ctx.accounts.fixed_raffle.raffle_id = ctx.accounts.raffle.key();

        Ok(())
    }

    pub fn close_raffle(ctx: Context<CloseRaffle>, force_close: bool) -> Result<()> {
        let is_admin = ctx.accounts.payer.key.to_string() == VLAWMZ_KEY && force_close;
        let ticket_account = ctx.accounts.fixed_raffle.to_account_info();
        let ticket_data = &mut ticket_account.data.borrow_mut();

        if ctx.accounts.fixed_raffle.owner != &ID || &ticket_data[8..40] != ctx.accounts.raffle.id.as_ref() {
            return err!(CustomError::InputError);
        }

        if force_close && !is_admin {
            return err!(CustomError::InputError);
        }

        let raffle = &ctx.accounts.raffle;

        if raffle.sent_out == 0 && raffle.tickets_purchased > 0 && !is_admin {
            return err!(CustomError::RaffleStarted);
        }

        // they need to pay out all winners
        if raffle.winners.len() != 0 && !is_admin {
            return err!(CustomError::CantScam);
        }

        let seeds: &[&[_]] = &[&[
            ctx.accounts.raffle.owner.as_ref(),
            ctx.accounts.mint_cost.to_account_info().key.as_ref(),
            ctx.accounts.mint_prize.to_account_info().key.as_ref(),
            &[raffle.bump]
        ]];

        let prize_decimals = (ctx.accounts.mint_prize.decimals - raffle.prize_decimals) as u32;
        let cost_decimals = (ctx.accounts.mint_cost.decimals - raffle.cost_decimals) as u32;

        // take prize tokens back from escrow
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                    from:  ctx.accounts.escrow_token_prize.to_account_info(),
                    to: ctx.accounts.token_prize.to_account_info(),
                    authority:  ctx.accounts.raffle.to_account_info()
                },
                seeds
            ),
            // draw back the prize tokens if there are any left over
            (raffle.prize_quantity - (raffle.per_win * raffle.sent_out as u64)) * 10_u64.pow(prize_decimals),
        )?;

        if raffle.burn && !is_admin {
            // burn
            anchor_spl::token::burn(
                CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Burn {
                        mint: ctx.accounts.mint_cost.to_account_info(),
                        from:  ctx.accounts.escrow_token_cost.to_account_info(),
                        authority:  ctx.accounts.raffle.to_account_info()
                    },
                    seeds
                ),
                raffle.tickets_purchased * 10_u64.pow(cost_decimals),
            )?;
        } else {
            // take paid tokens back from escrow
            anchor_spl::token::transfer(
                CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                        from:  ctx.accounts.escrow_token_cost.to_account_info(),
                        to: ctx.accounts.token_cost.to_account_info(),
                        authority:  ctx.accounts.raffle.to_account_info()
                    },
                    seeds
                ),
                raffle.tickets_purchased * 10_u64.pow(cost_decimals),
            )?;
        }

        anchor_spl::token::close_account(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::CloseAccount {
                    account: ctx.accounts.escrow_token_cost.to_account_info(),
                    destination: ctx.accounts.payer.to_account_info(),
                    authority: ctx.accounts.raffle.to_account_info()
                },
                seeds
            ),
        )?;

        anchor_spl::token::close_account(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::CloseAccount {
                    account: ctx.accounts.escrow_token_cost.to_account_info(),
                    destination: ctx.accounts.vlawmz.to_account_info(),
                    authority: ctx.accounts.raffle.to_account_info()
                },
                seeds
            ),
        )?;

        let raffle = ctx.accounts.raffle.to_account_info();
        let fixed_raffle = ctx.accounts.fixed_raffle.to_account_info();

        let payer = ctx.accounts.payer.to_account_info();
        let vlawmz = ctx.accounts.vlawmz.to_account_info();

        let mut escrow_lams = raffle.lamports.borrow_mut();
        let mut fixed_raffle_lams = fixed_raffle.lamports.borrow_mut();

        if payer.key == vlawmz.key {
            let mut payer_lams  = payer.lamports.borrow_mut();
            **payer_lams += **escrow_lams;
            **payer_lams += **fixed_raffle_lams;
        } else {
            let mut vlawmz_lams = vlawmz.lamports.borrow_mut();
            let mut payer_lams  = payer.lamports.borrow_mut();

            **vlawmz_lams += **escrow_lams;
            **vlawmz_lams += **fixed_raffle_lams / 10;

            **payer_lams += **fixed_raffle_lams - **fixed_raffle_lams / 10;
        }

        **escrow_lams = 0;
        **fixed_raffle_lams = 0;

        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>, amount: u64) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle;
        let ticket_account = ctx.accounts.fixed_raffle.to_account_info();
        let ticket_data = &mut ticket_account.data.borrow_mut();

        if ctx.accounts.fixed_raffle.owner != &ID || &ticket_data[8..40] != raffle.id.as_ref() || raffle.owner == *ctx.accounts.payer.key {
            return err!(CustomError::InputError);
        }

        let clock = Clock::get()?;

        if amount > 1200 {
            return err!(CustomError::TooMany);
        }

        if clock.unix_timestamp > raffle.end {
            return err!(CustomError::TooLate);
        }

        if clock.unix_timestamp < raffle.start {
            return err!(CustomError::TooEarly);
        }

        if amount > raffle.max_entries - raffle.tickets_purchased || raffle.max_entries <= raffle.tickets_purchased {
            return err!(CustomError::NotEnough);
        }

        let payer_bytes = &ctx.accounts.payer.key().to_bytes()[..];

        let mut offset: usize = RAFFLE_ENTRY_OFFSET + RAFFLE_ENTRY_SIZE * raffle.tickets_purchased as usize;

        for _x in 0..amount {
            ticket_data[offset .. offset + 32].copy_from_slice(payer_bytes);
            offset = offset + RAFFLE_ENTRY_SIZE;
        };

        raffle.tickets_purchased = raffle.tickets_purchased + amount;

        let mut unique = BTreeSet::new();
        let mut unique_entries = 0;

        for offset in (RAFFLE_ENTRY_OFFSET..raffle.tickets_purchased as usize).step_by(RAFFLE_ENTRY_SIZE) {
            let payer_slice = &ticket_data[offset..offset + 8];

            if !unique.contains(&payer_slice) {
                unique_entries = unique_entries + 1;
                unique.insert(payer_slice);
            }

            if offset > 15000 {
                break;
            }
        }

        raffle.unique_entries = unique_entries;

        let cost_decimals = (ctx.accounts.mint_cost.decimals - raffle.cost_decimals) as u32;

        anchor_spl::token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                from: ctx.accounts.token_cost.to_account_info(),
                to:  ctx.accounts.escrow_token_cost.to_account_info(),
                authority:  ctx.accounts.payer.to_account_info()
            }),
            raffle.price * 10_u64.pow(cost_decimals) * amount,
        )?;

        // vec size
        ticket_data[RAFFLE_ENTRY_OFFSET - 4 .. RAFFLE_ENTRY_OFFSET].copy_from_slice(&raffle.tickets_purchased.to_le_bytes()[..4]);

        Ok(())
    }

    pub fn draw_winner(ctx: Context<DrawWinner>) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle;
        let ticket_account = ctx.accounts.fixed_raffle.to_account_info();
        let ticket_data = &mut ticket_account.data.borrow_mut();

        if ctx.accounts.fixed_raffle.owner != &ID || &ticket_data[8..40] != raffle.id.as_ref() {
            return err!(CustomError::InputError);
        }

        // all winners need to be set first
        if raffle.winners_selected == false {
            return err!(CustomError::InputError);
        }

        if raffle.winners.len() == 0 {
            return err!(CustomError::AllWinnersPaid);
        }

        let winner = raffle.winners.pop().unwrap() as usize;

        let offset = RAFFLE_ENTRY_OFFSET + RAFFLE_ENTRY_SIZE * winner;

        if ctx.accounts.recipient.key.as_ref() != &ticket_data[offset .. offset + 32] {
            return err!(CustomError::InputError);
        }

        let seeds: &[&[_]] = &[&[
            raffle.owner.as_ref(),
            ctx.accounts.mint_cost.to_account_info().key.as_ref(),
            ctx.accounts.mint_prize.to_account_info().key.as_ref(),
            &[raffle.bump]
        ]];

        let prize_decimals = (ctx.accounts.mint_prize.decimals - raffle.prize_decimals) as u32;

        anchor_spl::token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                    from: ctx.accounts.escrow_token_prize.to_account_info(),
                    to: ctx.accounts.token_prize.to_account_info(),
                    authority: raffle.to_account_info()
                },
                seeds
            ),
            raffle.per_win * 10_u64.pow(prize_decimals),
        )?;

        // incremenet this so we can close a raffle just in case
        raffle.sent_out = raffle.sent_out + 1;

        Ok(())
    }

    pub fn set_winner(ctx: Context<SetWinner>) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle;
        let ticket_account = ctx.accounts.fixed_raffle.to_account_info();
        let ticket_data = &mut ticket_account.data.borrow_mut();

        if ctx.accounts.fixed_raffle.owner != &ID || &ticket_data[8..40] != raffle.id.as_ref() {
            return err!(CustomError::InputError);
        }

        let clock = Clock::get()?;

        if clock.unix_timestamp < raffle.end && raffle.max_entries > raffle.tickets_purchased {
            return err!(CustomError::RaffleGoing);
        }

        let slot_hashes = &ctx.accounts.slot_hashes;

        if slot_hashes.key().to_string() != "SysvarS1otHashes111111111111111111111111111" || raffle.tickets_purchased == 0 {
            return err!(CustomError::InputError);
        }

        let random = u64::from_le_bytes(slot_hashes.to_account_info().data.borrow()[16..24].try_into().unwrap());
        let winner: usize = random.checked_rem(raffle.tickets_purchased).unwrap() as usize;

        // we have reached a max # of winners and can not set anymore
        if raffle.winners_selected {
            return err!(CustomError::WinnersAlreadyPicked);
        }

        let offset = RAFFLE_ENTRY_OFFSET + RAFFLE_ENTRY_SIZE * winner;

        // this entry has 'won' already and can't win multiple times
        if ticket_data[offset + 32] >= 1 && !raffle.win_multiple {
            return err!(CustomError::InputError);
        }

        // we can count wins I guess up to 255? not an issue (?)
        ticket_data[offset + 32] = ticket_data[offset + 32] + 1;

        raffle.winners.push(winner as u64);

        // all winners have been picked, we can now pay them out
        if raffle.winners.len() == (raffle.prize_quantity / raffle.per_win) as usize {
            raffle.winners_selected = true;
        }

        Ok(())
    }

    pub fn init_token_accounts(ctx: Context<InitTokenAccounts>) -> Result<()> {
        if ctx.accounts.raffle.to_account_info().data.borrow().len() > 0 && ctx.accounts.raffle.owner != &ID {
            return err!(CustomError::InputError);
        }

        if ctx.accounts.escrow_token_prize.to_account_info().data.borrow().len() == 0 {
            anchor_spl::associated_token::create(
                CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    associated_token: ctx.accounts.escrow_token_prize.to_account_info(),
                    authority: ctx.accounts.raffle.to_account_info(),
                    mint: ctx.accounts.mint_prize.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info()
                }),
            )?;
        }

        if ctx.accounts.token_prize.to_account_info().data.borrow().len() == 0 {
            anchor_spl::associated_token::create(
                CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    associated_token: ctx.accounts.token_prize.to_account_info(),
                    authority: ctx.accounts.recipient.to_account_info(),
                    mint: ctx.accounts.mint_prize.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info()
                }),
            )?;
        }

        if ctx.accounts.escrow_token_cost.to_account_info().data.borrow().len() == 0 {
            anchor_spl::associated_token::create(
                CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    associated_token: ctx.accounts.escrow_token_cost.to_account_info(),
                    authority: ctx.accounts.raffle.to_account_info(),
                    mint: ctx.accounts.mint_cost.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info()
                }),
            )?;
        }

        if ctx.accounts.token_cost.to_account_info().data.borrow().len() == 0 {
            anchor_spl::associated_token::create(
                CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::associated_token::Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    associated_token: ctx.accounts.token_cost.to_account_info(),
                    authority: ctx.accounts.recipient.to_account_info(),
                    mint: ctx.accounts.mint_cost.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info()
                }),
            )?;
        }

        Ok(())
    }
}
