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
    TooLate,

    #[msg("Raffle has started!")]
    RaffleStarted,

    #[msg("The raffle has yet to start!")]
    TooEarly,

    #[msg("You can't buy more than 1000 at a time")]
    TooMany,

    #[msg("The raffle is still going!")]
    RaffleGoing,

    #[msg("Winners already picked")]
    WinnersAlreadyPicked,

    #[msg("All Winners Paid")]
    AllWinnersPaid,

    #[msg("You need to finish paying out the winners!")]
    CantScam,

    #[msg("The time parameters don't add up.")]
    TimeError,

    #[msg("later")]
    FixedError,

    #[msg("The payment needs to come to an even number!")]
    DivisibleError,

    #[msg("The payment needs to come to an even number!")]
    DecimalError
}
