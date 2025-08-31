//lendingProtocol-feature.tsx
'use client'

import { useConnection, useWallet } from '@solana/wallet-adapter-react'
import { WalletButton } from '../solana/solana-provider'
import { AppHero } from '../app-hero'
import { useLendingProgram } from './lendingProtocol-data-access'
import { GovernanceCard, AssetPoolList } from './lendingProtocol-ui'
import { ellipsify } from '@/lib/utils'
import { ExplorerLink } from '../cluster/cluster-ui'
import { useEffect, useState } from 'react'
import { LAMPORTS_PER_SOL } from '@solana/web3.js'
import { useQuery } from '@tanstack/react-query'

export default function LendingFeature() {
  const { publicKey } = useWallet()
  const { connection } = useConnection()
  const { programId } = useLendingProgram()

  // --- 核心修复：使用 useQuery 来管理余额 ---
  const { data: balance, isLoading: isBalanceLoading } = useQuery({
    queryKey: ['sol-balance', { wallet: publicKey?.toBase58() }],
    queryFn: () => connection.getBalance(publicKey!),
    enabled: !!publicKey, // 只有在连接钱包后才执行
  })
  return publicKey ? (
    <div>
      <AppHero
        title="Solana Lending Protocol"
        subtitle="A decentralized lending and borrowing platform built on Anchor."
      >
        <p className="mb-2 font-bold">Wallet: {publicKey.toBase58()}</p>
        <p className="mb-6 font-bold">
          Balance:{' '}
          {isBalanceLoading
            ? 'Loading...'
            : balance !== undefined && balance !== null
              ? `${(balance / LAMPORTS_PER_SOL).toFixed(4)} SOL`
              : 'Not available'}
        </p>
        <p className="mb-6">
          Program ID: <ExplorerLink path={`account/${programId}`} label={ellipsify(programId.toString())} />
        </p>
      </AppHero>
      <div className="space-y-8">
        <GovernanceCard />
        <AssetPoolList />
      </div>
    </div>
  ) : (
    <div className="max-w-4xl mx-auto">
      <div className="hero py-[64px]">
        <div className="hero-content text-center">
          <WalletButton />
        </div>
      </div>
    </div>
  )
}
