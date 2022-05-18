// client.js is used to introduce the reader to generating clients from IDLs.
// It is not expected users directly test with this example. For a more
// ergonomic example, see `tests/basic-0.js` in this workspace.

const anchor = require('@project-serum/anchor');
const fs = require('fs');
require('dotenv').config();

process.env.ANCHOR_WALLET = process.env.wallet;

let connection;
// Configure the local cluster.
connection = new anchor.web3.Connection(process.env.rpc);
anchor.setProvider(anchor.AnchorProvider.local(process.env.rpc));

async function testCreateRaffle(bad_params) {
  const mint = new anchor.web3.PublicKey(
      'meebAU3nZrU5PbUt3dVK6ExgbNWCUAkV7C3DaJKMZZ4',
    ),
    mintPrize = new anchor.web3.PublicKey(
      'ankhim7kPXxLKVbW1Tn7vH4mLTuvCAqHjhkKuvwWJ7b',
    ),
    [raffle, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [payer.wallet.publicKey.toBytes(), mint.toBytes(), mintPrize.toBytes()],
      programId,
    );

  const systemProgram = new anchor.web3.PublicKey(
      '11111111111111111111111111111111',
    ),
    rent = new anchor.web3.PublicKey(
      'SysvarRent111111111111111111111111111111111',
    ),
    escrowToken = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: raffle,
    }),
    escrowTokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: raffle,
    }),
    tokenPrize = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: payer.wallet.publicKey,
    }),
    tokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: payer.wallet.publicKey,
    });

  const spl_token_program = anchor.Spl.token();

  let tx = new anchor.web3.Transaction();

  if (
    (await payer.connection.getAccountInfo(escrowToken)) === null ||
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
        escrowTokenPrize: escrowToken,
        escrowTokenCost: escrowTokenCost,
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
    end: new anchor.BN(93999999990),
    maxEntries: new anchor.BN(99999),
    perWin: new anchor.BN(1),
    winMultiple: true,
    burn: false,
    fixed: true,
    description: 'AAAAAAAAAAAAAAAAAAA',
    nftUri: 'AAAAAAAAAAAAAAAAAAA',
    nftImage: 'AAAAAAAAAAAAAAAAAAA',
  };

  if (bad_params) {
    args.quantity = new anchor.BN(0);
  }

  const fixedRaffle = await anchor.web3.Keypair.fromSeed(
    new Uint8Array(raffle.toBytes()),
  );

  const ctx = {
    accounts: {
      payer: payer.wallet.publicKey,
      mint,
      tokenPrize,
      mintPrize,
      raffle,
      systemProgram,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      escrowToken,
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
  const mint = new anchor.web3.PublicKey(
      'meebAU3nZrU5PbUt3dVK6ExgbNWCUAkV7C3DaJKMZZ4',
    ),
    mintPrize = new anchor.web3.PublicKey(
      'ankhim7kPXxLKVbW1Tn7vH4mLTuvCAqHjhkKuvwWJ7b',
    ),
    tokenPrize = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: payer.wallet.publicKey,
    }),
    tokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: payer.wallet.publicKey,
    }),
    [raffle, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [payer.wallet.publicKey.toBytes(), mint.toBytes(), mintPrize.toBytes()],
      programId,
    ),
    fixedRaffle = await anchor.web3.Keypair.fromSeed(
      new Uint8Array(raffle.toBytes()),
    );

  const systemProgram = new anchor.web3.PublicKey(
    '11111111111111111111111111111111',
  );
  const escrowTokenPrize = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: raffle,
    }),
    escrowTokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: raffle,
    });

  const ctx = {
    accounts: {
      payer: payer.wallet.publicKey,
      mint,
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

  let a = await program.rpc.closeRaffle(force_close, ctx);
  return a;
}

async function testBuyRaffle() {
  const mint = new anchor.web3.PublicKey(
      'meebAU3nZrU5PbUt3dVK6ExgbNWCUAkV7C3DaJKMZZ4',
    ),
    tokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: buyer.publicKey,
    }),
    mintPrize = new anchor.web3.PublicKey(
      'ankhim7kPXxLKVbW1Tn7vH4mLTuvCAqHjhkKuvwWJ7b',
    ),
    [raffle, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [payer.wallet.publicKey.toBytes(), mint.toBytes(), mintPrize.toBytes()],
      programId,
    ),
    fixedRaffle = await anchor.web3.Keypair.fromSeed(
      new Uint8Array(raffle.toBytes()),
    );

  const systemProgram = new anchor.web3.PublicKey(
      '11111111111111111111111111111111',
    ),
    escrowTokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: raffle,
    });

  const ctx = {
    accounts: {
      payer: buyer.publicKey,
      mint,
      tokenCost,
      mintPrize,
      raffle,
      systemProgram,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      escrowTokenCost,
      fixedRaffle: fixedRaffle.publicKey,
    },
  };

  let sig = await buyer_program.rpc.buyTicket(new anchor.BN(1), ctx);
  anchor.setProvider(anchor.AnchorProvider.local(process.env.rpc));
  return sig;
}

const raffler_idl = JSON.parse(
    fs.readFileSync('./target/idl/raffler_anchor.json', 'utf8'),
  ),
  programId = new anchor.web3.PublicKey(
    '3XsaSBCDT4JhRuxpWjHRTYkzKLqWRgCuN1wyggvFuSsM',
  ),
  program = new anchor.Program(raffler_idl, programId),
  payer = program.provider,
  buyer = new anchor.Wallet(
    anchor.web3.Keypair.fromSecretKey(
      new Uint8Array([
        104, 71, 113, 233, 97, 67, 75, 109, 101, 145, 53, 155, 133, 64, 98, 233,
        1, 162, 226, 147, 78, 178, 35, 134, 253, 189, 127, 244, 200, 24, 150,
        135, 6, 226, 162, 242, 159, 83, 97, 123, 251, 176, 52, 102, 191, 184,
        183, 153, 186, 64, 236, 4, 79, 187, 154, 37, 7, 51, 240, 157, 234, 211,
        252, 137,
      ]),
    ),
  ),
  buyer_provider = new anchor.AnchorProvider(
    connection,
    buyer,
    anchor.AnchorProvider.defaultOptions(),
  );

anchor.setProvider(buyer_provider);
const buyer_program = new anchor.Program(raffler_idl, programId);

(async () => {
  try {
    //  await testCreateBadParams();
    await testCreateAndClose();
    //await testCreateAndForceClose();
    // await testCreate();
    // console.log(await testBuyRaffle());
    // for (let i = 0; i < 50; i++) {
    //   console.log(await testBuyRaffle());
    // }
    // for (let i = 0; i < 10; i++) {
    //   console.log(await testPickWinner());
    // }
    console.log(await testSendWinner());
  } catch (e) {
    console.log(e);
    //
  }
})();

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

async function testCreateBadParams() {
  try {
    let [tx, signers] = await testCreateRaffle(true);
    let a = await payer.sendAndConfirm(tx, [...signers, payer.wallet.payer], {
      skipPreflight: true,
      commitment: 'confirmed',
    });
    console.log(a);
  } catch (e) {
    console.log(e);
  }
}

async function testCreateAndClose(force_close) {
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

  try {
    let sig = await testCloseRaffle(force_close);
    console.log('close', sig);
  } catch (e) {
    console.log(e);
  }
}

async function testCreateAndForceClose() {
  await testCreateAndClose(true);
}

async function testPickWinner() {
  const mint = new anchor.web3.PublicKey(
      'meebAU3nZrU5PbUt3dVK6ExgbNWCUAkV7C3DaJKMZZ4',
    ),
    mintPrize = new anchor.web3.PublicKey(
      'ankhim7kPXxLKVbW1Tn7vH4mLTuvCAqHjhkKuvwWJ7b',
    ),
    [raffle, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [payer.wallet.publicKey.toBytes(), mint.toBytes(), mintPrize.toBytes()],
      programId,
    ),
    slotHashes = new anchor.web3.PublicKey(
      'SysvarS1otHashes111111111111111111111111111',
    );

  const fixedRaffle = await anchor.web3.Keypair.fromSeed(
    new Uint8Array(raffle.toBytes()),
  );

  const ctx = {
    accounts: {
      payer: payer.wallet.publicKey,
      mint,
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
  const mint = new anchor.web3.PublicKey(
      'meebAU3nZrU5PbUt3dVK6ExgbNWCUAkV7C3DaJKMZZ4',
    ),
    mintPrize = new anchor.web3.PublicKey(
      'ankhim7kPXxLKVbW1Tn7vH4mLTuvCAqHjhkKuvwWJ7b',
    ),
    [raffle, bump] = await anchor.web3.PublicKey.findProgramAddress(
      [payer.wallet.publicKey.toBytes(), mint.toBytes(), mintPrize.toBytes()],
      programId,
    );

  const systemProgram = new anchor.web3.PublicKey(
      '11111111111111111111111111111111',
    ),
    rent = new anchor.web3.PublicKey(
      'SysvarRent111111111111111111111111111111111',
    ),
    escrowToken = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: raffle,
    }),
    escrowTokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: raffle,
    }),
    tokenPrize = await anchor.utils.token.associatedAddress({
      mint: mintPrize,
      owner: buyer.publicKey,
    }),
    tokenCost = await anchor.utils.token.associatedAddress({
      mint: mint,
      owner: buyer.publicKey,
    });

  const spl_token_program = anchor.Spl.token();

  let tx = new anchor.web3.Transaction();

  if (
    (await payer.connection.getAccountInfo(escrowToken)) === null ||
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
        escrowTokenPrize: escrowToken,
        escrowTokenCost: escrowTokenCost,
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
      mint,
      tokenPrize,
      mintPrize,
      raffle,
      systemProgram,
      tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      escrowTokenPrize: escrowToken,
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
