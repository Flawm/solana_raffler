import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { RafflerAnchor } from "../target/types/raffler_anchor";

describe("raffler_anchor", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.RafflerAnchor as Program<RafflerAnchor>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
