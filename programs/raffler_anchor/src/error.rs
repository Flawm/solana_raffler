use {
    anchor_lang::error_code,
};

#[error_code]
pub enum CustomError {
    #[msg("Error with input data")]
    InputError,

    #[msg("Can't sell this amount of tickets.")]
    NotEnough,

    #[msg("Raffle has ended!")]
    RaffleEnded,

    #[msg("Raffle has started!")]
    RaffleStarted,

    #[msg("The raffle has yet to start!")]
    TooEarly,

    #[msg("You can't buy more than 1000 at a time")]
    TooMany,
}
