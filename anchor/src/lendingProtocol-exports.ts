// Here we export some useful types and functions for interacting with the Anchor program.
import { AnchorProvider, Program } from '@coral-xyz/anchor'
import { Cluster, PublicKey } from '@solana/web3.js'
import LendingProtocolIDL from '../target/idl/lending_protocol.json'
import type { LendingProtocol } from '../target/types/lending_protocol'

// Re-export the generated IDL and type
export { LendingProtocol, LendingProtocolIDL }

// The programId is imported from the program IDL.
export const LENDINGPROTOCOL_PROGRAM_ID = new PublicKey(LendingProtocolIDL.address)

// This is a helper function to get the LendingProtocol Anchor program.
export function getLendingProtocolProgram(provider: AnchorProvider, address?: PublicKey): Program<LendingProtocol> {
  return new Program(
    { ...LendingProtocolIDL, address: address ? address.toBase58() : LendingProtocolIDL.address } as LendingProtocol,
    provider,
  )
}

// This is a helper function to get the program ID for the LendingProtocol program depending on the cluster.
export function getLendingProtocolProgramId(cluster: Cluster) {
  switch (cluster) {
    case 'devnet':
    case 'testnet':
      // This is the program ID for the LendingProtocol program on devnet and testnet.
      return new PublicKey('coUnmi3oBUtwtd9fjeAvSsJssXh5A5xyPbhpewyzRVF')
    case 'mainnet-beta':
    default:
      return LENDINGPROTOCOL_PROGRAM_ID
  }
}
