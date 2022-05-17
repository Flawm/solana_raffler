pub mod error;
pub mod state;

use {
    anchor_lang::prelude::*,
    crate::{error::*, state::*},
    std::collections::BTreeSet,
};

declare_id!("3XsaSBCDT4JhRuxpWjHRTYkzKLqWRgCuN1wyggvFuSsM");

#[program]
pub mod raffler_anchor {
    use super::*;

    pub fn create_raffle(ctx: Context<CreateRaffle>, data: CreateRaffleData) -> Result<()> {
        if data.start >= data.end || data.prize_quantity == 0 || data.price == 0 || ctx.accounts.token_prize.amount < data.prize_quantity || data.prize_quantity < data.per_win {
            return err!(CustomError::InputError);
        }

        // later
        if data.fixed == false {
            return err!(CustomError::InputError);
        }

        let raffle = &mut ctx.accounts.raffle;
        raffle.id = raffle.key();
        raffle.owner = *ctx.accounts.payer.key;
        raffle.mint = ctx.accounts.mint.key();
        raffle.prize = ctx.accounts.mint_prize.key();
        raffle.prize_quantity = data.prize_quantity;
        raffle.tickets_purchased = 0;
        raffle.price = data.price;
        raffle.start = data.start;
        raffle.end = data.end;
        raffle.ticket_count = 0;
        raffle.max_entries = data.max_entries;
        raffle.per_win = data.per_win;
        raffle.win_multiple = data.win_multiple;
        raffle.description = data.description;
        raffle.bump = *ctx.bumps.get("raffle").unwrap();
        raffle.burn = data.burn;
        raffle.fixed = data.fixed;

        anchor_spl::token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                from: ctx.accounts.token_prize.to_account_info(),
                to:  ctx.accounts.escrow_token.to_account_info(),
                authority:  ctx.accounts.payer.to_account_info()
            }),
            raffle.prize_quantity * 10_u64.pow(ctx.accounts.mint_prize.decimals as u32),
        )?;

        ctx.accounts.fixed_raffle.raffle_id = ctx.accounts.raffle.key();

        Ok(())
    }

    pub fn close_raffle(ctx: Context<CloseRaffle>, force_close: bool) -> Result<()> {
        let is_admin = ctx.accounts.payer.key.to_string() == VLAWMZ_KEY;
        let ticket_account = ctx.accounts.fixed_raffle.to_account_info();
        let ticket_data = &mut ticket_account.data.borrow_mut();

        if &ticket_data[8..40] != ctx.accounts.raffle.id.as_ref() {
            return err!(CustomError::InputError);
        }

        if force_close && !is_admin {
            return err!(CustomError::InputError);
        }

        let raffle = &ctx.accounts.raffle;

        if raffle.tickets_purchased > 0 && !is_admin {
            return err!(CustomError::RaffleStarted);
        }


        let seeds: &[&[_]] = &[&[
            ctx.accounts.raffle.owner.as_ref(),
            ctx.accounts.mint.to_account_info().key.as_ref(),
            ctx.accounts.mint_prize.to_account_info().key.as_ref(),
            &[raffle.bump]
        ]];

        // take prize tokens back from escrow
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                    from:  ctx.accounts.escrow_token_prize.to_account_info(),
                    to: ctx.accounts.token_prize.to_account_info(),
                    authority:  ctx.accounts.raffle.to_account_info()
                },
                seeds
            ),
            raffle.prize_quantity * 10_u64.pow(ctx.accounts.mint_prize.decimals as u32),
        )?;

        if raffle.burn && !is_admin {
            // burn
            anchor_spl::token::burn(
                CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Burn {
                        mint: ctx.accounts.mint.to_account_info(),
                        from:  ctx.accounts.escrow_token_cost.to_account_info(),
                        authority:  ctx.accounts.raffle.to_account_info()
                    },
                    seeds
                ),
                raffle.tickets_purchased * 10_u64.pow(ctx.accounts.mint_prize.decimals as u32),
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
                raffle.tickets_purchased * 10_u64.pow(ctx.accounts.mint_prize.decimals as u32),
            )?;
        }

        let raffle = ctx.accounts.raffle.to_account_info();
        let fixed_raffle = ctx.accounts.fixed_raffle.to_account_info();

        let payer = ctx.accounts.payer.to_account_info();

        let mut payer_lams  = payer.lamports.borrow_mut();
        let mut escrow_lams = raffle.lamports.borrow_mut();
        let mut fixed_raffle_lams = fixed_raffle.lamports.borrow_mut();
        **payer_lams += **escrow_lams;
        **escrow_lams = 0;
        **payer_lams += **fixed_raffle_lams;
        **fixed_raffle_lams = 0;

        Ok(())
    }

    pub fn buy_ticket(ctx: Context<BuyTicket>, amount: u64) -> Result<()> {
        let raffle = &mut ctx.accounts.raffle;
        let ticket_account = ctx.accounts.fixed_raffle.to_account_info();
        let ticket_data = &mut ticket_account.data.borrow_mut();

        if &ticket_data[8..40] != raffle.id.as_ref() {
            return err!(CustomError::InputError);
        }

        let clock = Clock::get()?;

        if amount > 1200 {
            return err!(CustomError::TooMany);
        }

        if clock.unix_timestamp > raffle.end {
            return err!(CustomError::RaffleEnded);
        }

        if clock.unix_timestamp < raffle.start {
            return err!(CustomError::TooEarly);
        }

        if amount > raffle.max_entries - raffle.tickets_purchased || raffle.max_entries <= raffle.tickets_purchased {
            return err!(CustomError::NotEnough);
        }

        let payer_bytes = &ctx.accounts.payer.key().to_bytes()[..];

        // starts at 44
        let mut offset: usize = 8 + 32 + 4 + 33 * raffle.tickets_purchased as usize;

        for _x in 0..amount {
            ticket_data[offset..offset + 32].copy_from_slice(payer_bytes);
            // skips by 33
            offset = offset + 33;
        };

        raffle.tickets_purchased = raffle.tickets_purchased + amount;

        let mut unique = BTreeSet::new();
        let mut unique_entries = 0;

        for offset in (44..raffle.tickets_purchased as usize).step_by(33) {
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

        anchor_spl::token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), anchor_spl::token::Transfer {
                from: ctx.accounts.token_cost.to_account_info(),
                to:  ctx.accounts.escrow_token_cost.to_account_info(),
                authority:  ctx.accounts.payer.to_account_info()
            }),
            raffle.price * 10_u64.pow(ctx.accounts.mint_prize.decimals as u32) * amount,
        )?;

        // 40 - 44 is the length of our vec
        ticket_data[40..44].copy_from_slice(&raffle.tickets_purchased.to_le_bytes()[..4]);

        Ok(())
    }

    pub fn draw_winner(ctx: Context<DrawWinner>) -> Result<()> {
        Ok(())
    }

    pub fn init_token_accounts(ctx: Context<InitTokenAccounts>) -> Result<()> {
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
                    authority: ctx.accounts.raffle.to_account_info(),
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
                    authority: ctx.accounts.raffle.to_account_info(),
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
