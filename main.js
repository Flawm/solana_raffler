(async () => {
  const anchor = require('@project-serum/anchor'),
    fs = require('fs');
  require('dotenv').config();

  process.env.ANCHOR_WALLET = process.env.wallet;

  const connection = new anchor.web3.Connection(process.env.rpc),
    mintCost = new anchor.web3.PublicKey(process.env.mint_cost),
    mintPrize = new anchor.web3.PublicKey(process.env.mint_prize),
    systemProgram = new anchor.web3.PublicKey(
      '11111111111111111111111111111111',
    ),
    rent = new anchor.web3.PublicKey(
      'SysvarRent111111111111111111111111111111111',
    ),
    spl_token_program = anchor.Spl.token();

  anchor.setProvider(anchor.AnchorProvider.local(process.env.rpc));

  const raffler_idl = JSON.parse(
      fs.readFileSync('./target/idl/raffler_anchor.json', 'utf8'),
    ),
    programId = new anchor.web3.PublicKey(
      'RafXcAJfB3wVzyH7cHuDqyVjYeQ4Qy8RjQEbCPvttK9',
    ),
    program = new anchor.Program(raffler_idl, programId),
    payer = program.provider,
    buyer = new anchor.Wallet(
      anchor.web3.Keypair.fromSecretKey(
        new Uint8Array([
          104, 71, 113, 233, 97, 67, 75, 109, 101, 145, 53, 155, 133, 64, 98,
          233, 1, 162, 226, 147, 78, 178, 35, 134, 253, 189, 127, 244, 200, 24,
          150, 135, 6, 226, 162, 242, 159, 83, 97, 123, 251, 176, 52, 102, 191,
          184, 183, 153, 186, 64, 236, 4, 79, 187, 154, 37, 7, 51, 240, 157,
          234, 211, 252, 137,
        ]),
      ),
    ),
    buyer_provider = new anchor.AnchorProvider(
      connection,
      buyer,
      anchor.AnchorProvider.defaultOptions(),
    );

  const [raffle, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [
        payer.wallet.publicKey.toBytes(),
        mintCost.toBytes(),
        mintPrize.toBytes(),
      ],
      programId,
    ),
    escrowTokenPrize = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: raffle,
    }),
    escrowTokenCost = await anchor.utils.token.associatedAddress({
      mint: mintCost,
      owner: raffle,
    }),
    tokenPrize = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: payer.wallet.publicKey,
    }),
    tokenCost = await anchor.utils.token.associatedAddress({
      mint: mintCost,
      owner: payer.wallet.publicKey,
    }),
    fixedRaffle = await anchor.web3.Keypair.fromSeed(
      new Uint8Array(raffle.toBytes()),
    ),
    buyerPrize = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: buyer.publicKey,
    }),
    buyerCost = await anchor.utils.token.associatedAddress({
      mint: mintCost,
      owner: buyer.publicKey,
    });

  anchor.setProvider(buyer_provider);
  const buyer_program = new anchor.Program(raffler_idl, programId);

  async function testCreateRaffle() {
    let tx = new anchor.web3.Transaction();

    if (
      (await payer.connection.getAccountInfo(escrowTokenPrize)) === null ||
      (await payer.connection.getAccountInfo(escrowTokenCost)) === null ||
      (await payer.connection.getAccountInfo(tokenPrize)) === null ||
      (await payer.connection.getAccountInfo(tokenCost)) === null
    ) {
      let ctx_accounts = {
        accounts: {
          payer: payer.wallet.publicKey,
          mintCost: mint,
          mintPrize,
          tokenPrize,
          tokenCost,
          escrowTokenPrize,
          escrowTokenCost,
          associatedToken: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          systemProgram,
          raffle,
          rent,
          recipient: payer.wallet.publicKey,
        },
      };

      tx.add(program.instruction.initTokenAccounts(ctx_accounts));
    }

    const args = {
      prizeQuantity: new anchor.BN(5),
      price: new anchor.BN(1),
      start: new anchor.BN(150),
      end: new anchor.BN(Date.now() / 1000 + 5),
      costDecimals: 1,
      prizeDecimals: 9,
      maxEntries: new anchor.BN(99999),
      perWin: new anchor.BN(1),
      winMultiple: true,
      burn: false,
      fixed: true,
      description: 'AAAAAAAAAAAAAAAAAAA',
      nftUri: 'AAAAAAAAAAAAAAAAAAA',
      nftImage: 'AAAAAAAAAAAAAAAAAAA',
    };

    const ctx = {
      accounts: {
        payer: payer.wallet.publicKey,
        mintCost,
        tokenPrize,
        mintPrize,
        raffle,
        systemProgram,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        escrowTokenPrize,
        fixedRaffle: fixedRaffle.publicKey,
      },
    };

    tx.add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: payer.wallet.publicKey,
        newAccountPubkey: fixedRaffle.publicKey,
        programId: programId,
        lamports: await payer.connection.getMinimumBalanceForRentExemption(
          8 + 32 + 4 + 35 * args.maxEntries,
        ),
        space: 8 + 32 + 4 + 35 * args.maxEntries,
      }),
    );
    tx.add(await program.instruction.createRaffle(args, ctx));

    tx.setSigners(payer.wallet.publicKey, fixedRaffle.publicKey);

    return [tx, [fixedRaffle]];
  }

  async function testCloseRaffle(force_close) {
    const ctx = {
      accounts: {
        payer: payer.wallet.publicKey,
        mintCost,
        tokenPrize,
        tokenCost,
        mintPrize,
        raffle,
        systemProgram,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        escrowTokenPrize,
        escrowTokenCost,
        fixedRaffle: fixedRaffle.publicKey,
        vlawmz: new anchor.web3.PublicKey(
          'VLawmZTgLAbdeqrU579ohsdey9H1h3Mi1UeUJpg2mQB',
        ),
      },
    };

    return await program.rpc.closeRaffle(force_close, ctx);
  }

  async function testBuyRaffle() {
    const buyerCost = await anchor.utils.token.associatedAddress({
      mint: mintCost,
      owner: buyer.publicKey,
    });

    const ctx = {
      accounts: {
        payer: buyer.publicKey,
        mintCost,
        tokenCost: buyerCost,
        mintPrize,
        raffle,
        systemProgram,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        escrowTokenCost,
        fixedRaffle: fixedRaffle.publicKey,
      },
    };

    return await buyer_program.rpc.buyTicket(new anchor.BN(1), ctx);
  }

  async function testPickWinner() {
    const slotHashes = new anchor.web3.PublicKey(
      'SysvarS1otHashes111111111111111111111111111',
    );

    const ctx = {
      accounts: {
        payer: payer.wallet.publicKey,
        mintCost,
        mintPrize,
        raffle,
        fixedRaffle: fixedRaffle.publicKey,
        slotHashes,
      },
    };

    let a = await program.rpc.setWinner(ctx);
    return a;
  }

  async function testSendWinner() {
    let tx = new anchor.web3.Transaction();

    if (
      (await payer.connection.getAccountInfo(escrowTokenPrize)) === null ||
      (await payer.connection.getAccountInfo(escrowTokenCost)) === null ||
      (await payer.connection.getAccountInfo(buyerPrize)) === null ||
      (await payer.connection.getAccountInfo(buyerCost)) === null
    ) {
      let ctx_accounts = {
        accounts: {
          payer: payer.wallet.publicKey,
          mintCost: mintCost,
          mintPrize,
          tokenPrize: buyerPrize,
          tokenCost: buyerCost,
          escrowTokenPrize,
          escrowTokenCost,
          associatedToken: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          rent,
          raffle,
          recipient: buyer.publicKey,
        },
      };

      tx.add(program.instruction.initTokenAccounts(ctx_accounts));
    }

    const fixedRaffle = await anchor.web3.Keypair.fromSeed(
      new Uint8Array(raffle.toBytes()),
    );

    const ctx = {
      accounts: {
        payer: payer.wallet.publicKey,
        recipient: buyer.publicKey,
        mintCost,
        tokenPrize: buyerPrize,
        mintPrize,
        raffle,
        systemProgram,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        escrowTokenPrize,
        fixedRaffle: fixedRaffle.publicKey,
      },
    };

    tx.add(await program.instruction.drawWinner(ctx));

    tx.setSigners(payer.wallet.publicKey);

    let a = await payer.sendAndConfirm(tx, [payer.wallet.payer], {
      skipPreflight: true,
      commitment: 'confirmed',
    });
    return a;
  }

  async function testCreate() {
    try {
      let [tx, signers] = await testCreateRaffle();
      let a = await payer.sendAndConfirm(tx, [...signers, payer.wallet.payer], {
        skipPreflight: true,
        commitment: 'confirmed',
      });
      console.log(a);
    } catch (e) {
      console.log(e);
    }
  }

  (async () => {
    try {
      await testCreate();
      for (let i = 0; i < 3; i++) {
        try {
          console.log(await testBuyRaffle());
        } catch (e) {
          console.log(e);
          break;
        }
      }
      await new Promise((r) => setTimeout(r, 5000));
      for (let i = 0; i < 10; i++) {
        try {
          console.log(await testPickWinner());
        } catch (e) {
          console.log(e);
          break;
        }
      }
      for (let i = 0; i < 10; i++) {
        try {
          console.log(await testSendWinner());
        } catch (e) {
          console.log(e);
          break;
        }
      }
      console.log(await testCloseRaffle());
    } catch (e) {
      console.log(e);
      //
    }
  })();
})();
