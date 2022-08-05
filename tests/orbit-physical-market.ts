import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { OrbitPhysicalMarket } from "../target/types/orbit_physical_market";

describe("orbit-physical-market", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.OrbitPhysicalMarket as Program<OrbitPhysicalMarket>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
