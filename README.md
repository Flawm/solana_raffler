# Solana Raffler

A Solana raffling program built on Anchor. Deployed at `RafXcAJfB3wVzyH7cHuDqyVjYeQ4Qy8RjQEbCPvttK9` on devnet - mainnet coming soon :tm:

## Features

- Supports all tokens. FT, SFT, NFT - anything that is an SPL token. You can raffle USDC for wSol, whitelist tokens for an NFT - anything.
- Variable configuration, including a burn setting if you're intending the raffle to be deflationary
- Supports multiple winners with variable payout setting, ie you're raffling 10 white list tokens at once - you can pay out 2 to 5 winners, or 1 to 10 winners.

## Testing

- .env file should have `rpc`, `wallet`, `mint_cost`, and `mint_prize`. Fund the TST wallet make sure it has the cost tokens
