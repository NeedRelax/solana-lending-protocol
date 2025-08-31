'use client'

import { Program, BN } from '@coral-xyz/anchor'
import {
  LendingProtocol,
  LendingProtocolIDL,
  getLendingProtocolProgramId,
  getLendingProtocolProgram,
} from '@project/anchor'
import { useConnection, useWallet } from '@solana/wallet-adapter-react'
import { Cluster, Keypair, PublicKey, SystemProgram } from '@solana/web3.js'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useMemo } from 'react'
import { toast } from 'sonner'
import { useCluster } from '../cluster/cluster-data-access'
import { useAnchorProvider } from '../solana/solana-provider'
import { useTransactionToast } from '../use-transaction-toast'
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync, getMint, getAccount } from '@solana/spl-token'

// --- 核心 Program Hook ---
export function useLendingProgram() {
  const { cluster } = useCluster()
  const provider = useAnchorProvider()
  const transactionToast = useTransactionToast()
  const queryClient = useQueryClient()

  const programId = useMemo(() => getLendingProtocolProgramId(cluster.network as Cluster), [cluster]) // 基于集群计算程序 ID
  const program = useMemo(() => getLendingProtocolProgram(provider), [provider]) // 获取 Pumpfun 程序实例

  const [marketConfigPda] = useMemo(
    () => PublicKey.findProgramAddressSync([Buffer.from('market_config')], program.programId),
    [program.programId],
  )

  const marketConfigAccount = useQuery({
    queryKey: ['lending', 'marketConfig', { cluster }],
    queryFn: async () => {
      try {
        return await program.account.marketConfig.fetch(marketConfigPda)
      } catch (e) {
        console.log('Failed to fetch market config, it might not be initialized yet.', e)
        return null
      }
    },
  })

  const assetPools = useQuery({
    queryKey: ['lending', 'assetPools', { cluster }],
    queryFn: async () => {
      if (!marketConfigAccount.data) return []
      const poolAddresses = marketConfigAccount.data.pools.slice(0, marketConfigAccount.data.poolCount)
      if (poolAddresses.length === 0) return []
      return program.account.assetPool.fetchMultiple(poolAddresses)
    },
    enabled: !!marketConfigAccount.data,
  })

  const onTransactionSuccess = (signature: string) => {
    transactionToast(signature)
    toast.success('Transaction successful!')
    queryClient.invalidateQueries({ queryKey: ['lending', 'marketConfig'] })
    queryClient.invalidateQueries({ queryKey: ['user-balance'] })
  }

  const initializeMarket = useMutation({
    mutationKey: ['lending', 'initializeMarket', { cluster }],
    mutationFn: () =>
      program.methods
        .initializeMarketConfig()
        .accounts({
          marketConfig: marketConfigPda,
          owner: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
    onSuccess: onTransactionSuccess,
    onError: (err: Error) => toast.error(`Failed to initialize market: ${err.message}`),
  })

  const addAssetPoolMutation = useMutation({
    mutationKey: ['lending', 'addAssetPool', { cluster }],
    mutationFn: async ({ assetMint, pythPriceFeed }: { assetMint: PublicKey; pythPriceFeed: PublicKey }) => {
      const assetPoolKeypair = Keypair.generate()
      const [assetVaultPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('asset_vault'), assetPoolKeypair.publicKey.toBuffer()],
        program.programId,
      )
      const params = {
        loanToValueBps: new BN(8000),
        liquidationThresholdBps: new BN(8500),
        baseBorrowRateBps: new BN(100),
        baseSlopeBps: new BN(500),
        optimalUtilizationBps: new BN(8000),
        kinkSlopeBps: new BN(2000),
        protocolFeeBps: new BN(1000),
        flashLoanFeeBps: new BN(25),
      }

      return program.methods
        .addAssetPool(params)
        .accounts({
          marketConfig: marketConfigPda,
          assetPool: assetPoolKeypair.publicKey,
          assetVault: assetVaultPda,
          assetMint: assetMint,
          governanceAuthority: provider.wallet.publicKey,
          pythPriceFeedAccount: pythPriceFeed,
          chainlinkPriceFeedAccount: null,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([assetPoolKeypair])
        .rpc()
    },
    onSuccess: onTransactionSuccess,
    onError: (err: Error) => toast.error(`Failed to add asset pool: ${err.message}`),
  })

  return {
    program,
    programId,
    marketConfigPda,
    marketConfigAccount,
    assetPools,
    initializeMarket,
    addAssetPoolMutation,
  }
}

// --- Hook for interacting with a specific Asset Pool and User Position ---
export function useLendingPool({ poolAddress }: { poolAddress: PublicKey }) {
  const { connection } = useConnection()
  const { cluster } = useCluster()
  const wallet = useWallet()
  const transactionToast = useTransactionToast()
  const { program, marketConfigPda } = useLendingProgram()
  const queryClient = useQueryClient()

  const poolAccount = useQuery({
    queryKey: ['lending', 'pool', { cluster, poolAddress }],
    queryFn: () => program.account.assetPool.fetch(poolAddress),
  })

  const mintDecimalsQuery = useQuery({
    queryKey: ['lending', 'mintDecimals', { cluster, mint: poolAccount.data?.assetMint.toBase58() }],
    queryFn: async () => {
      if (!poolAccount.data?.assetMint) return null
      const mintInfo = await getMint(connection, poolAccount.data.assetMint)
      return mintInfo.decimals
    },
    enabled: !!poolAccount.data?.assetMint,
  })

  const [userPositionPda] = useMemo(
    () =>
      wallet.publicKey
        ? PublicKey.findProgramAddressSync(
            [Buffer.from('user_position'), poolAddress.toBuffer(), wallet.publicKey.toBuffer()],
            program.programId,
          )
        : [null],
    [program.programId, poolAddress, wallet.publicKey],
  )

  const userPositionAccount = useQuery({
    queryKey: ['lending', 'userPosition', { cluster, poolAddress, user: wallet.publicKey?.toBase58() }],
    queryFn: async () => {
      if (!userPositionPda) return null
      try {
        return await program.account.userPosition.fetch(userPositionPda)
      } catch (e) {
        return null
      }
    },
    enabled: !!wallet.publicKey && !!userPositionPda,
  })

  const getUserAta = (mint: PublicKey) => {
    if (!wallet.publicKey) throw new Error('Wallet not connected')
    return getAssociatedTokenAddressSync(mint, wallet.publicKey)
  }

  const onSuccess = (signature: string) => {
    transactionToast(signature)
    toast.success('Transaction successful!')
    queryClient.invalidateQueries({ queryKey: ['lending', 'userPosition', { cluster, poolAddress }] })
    queryClient.invalidateQueries({ queryKey: ['lending', 'pool', { cluster, poolAddress }] })
    queryClient.invalidateQueries({ queryKey: ['user-balance', { mint: poolAccount.data?.assetMint.toBase58() }] })
  }

  const onError = (err: Error) => toast.error(`Transaction failed: ${err.message}`)

  const createUserPosition = useMutation({
    mutationKey: ['lending', 'createUserPosition', { cluster, poolAddress, user: wallet.publicKey }],
    mutationFn: () =>
      program.methods
        .createUserPosition()
        .accounts({
          userPosition: userPositionPda,
          user: wallet.publicKey!,
          assetPool: poolAddress,
          systemProgram: SystemProgram.programId,
        })
        .rpc(),
    onSuccess: (sig) => {
      transactionToast(sig)
      userPositionAccount.refetch()
    },
    onError: (err: Error) => toast.error(`Failed to create position: ${err.message}`),
  })

  const depositMutation = useMutation({
    mutationKey: ['lending', 'deposit', { cluster, poolAddress, user: wallet.publicKey }],
    mutationFn: async (amount: number) => {
      const decimals = mintDecimalsQuery.data
      if (!poolAccount.data || !wallet.publicKey || decimals === undefined || decimals === null)
        throw new Error('Data not ready')
      const amountBN = new BN(amount * 10 ** decimals)
      const userAssetAccount = getUserAta(poolAccount.data.assetMint)
      return program.methods
        .deposit(amountBN)
        .accounts({
          marketConfig: marketConfigPda,
          assetPool: poolAddress,
          userPosition: userPositionPda,
          user: wallet.publicKey,
          userAssetAccount,
          assetVault: poolAccount.data.assetVault,
          assetMint: poolAccount.data.assetMint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc()
    },
    onSuccess,
    onError,
  })

  const repayMutation = useMutation({
    mutationKey: ['lending', 'repay', { cluster, poolAddress, user: wallet.publicKey }],
    mutationFn: async (amount: number) => {
      const decimals = mintDecimalsQuery.data
      if (!poolAccount.data || !wallet.publicKey || decimals === undefined || decimals === null)
        throw new Error('Data not ready')
      const amountBN = new BN(amount * 10 ** decimals)
      const userAssetAccount = getUserAta(poolAccount.data.assetMint)
      return program.methods
        .repay(amountBN)
        .accounts({
          marketConfig: marketConfigPda,
          assetPool: poolAddress,
          userPosition: userPositionPda,
          user: wallet.publicKey,
          userAssetAccount,
          assetVault: poolAccount.data.assetVault,
          assetMint: poolAccount.data.assetMint,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .rpc()
    },
    onSuccess,
    onError,
  })

  const withdrawMutation = useMutation({
    mutationKey: ['lending', 'withdraw', { cluster, poolAddress, user: wallet.publicKey }],
    mutationFn: async (amount: number) => {
      const decimals = mintDecimalsQuery.data
      if (!poolAccount.data || !wallet.publicKey || decimals === undefined || decimals === null)
        throw new Error('Data not ready')
      const amountBN = new BN(amount * 10 ** decimals)
      const userAssetAccount = getUserAta(poolAccount.data.assetMint)
      return program.methods
        .withdraw(amountBN)
        .accounts({
          marketConfig: marketConfigPda,
          assetPool: poolAddress,
          userPosition: userPositionPda,
          user: wallet.publicKey,
          userAssetAccount,
          assetVault: poolAccount.data.assetVault,
          assetMint: poolAccount.data.assetMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          pythPriceFeedAccount: poolAccount.data.pythPriceFeed,
          chainlinkPriceFeedAccount: SystemProgram.programId,
        })
        .rpc()
    },
    onSuccess,
    onError,
  })

  const borrowMutation = useMutation({
    mutationKey: ['lending', 'borrow', { cluster, poolAddress, user: wallet.publicKey }],
    mutationFn: async (amount: number) => {
      const decimals = mintDecimalsQuery.data
      if (!poolAccount.data || !wallet.publicKey || decimals === undefined || decimals === null)
        throw new Error('Data not ready')
      const amountBN = new BN(amount * 10 ** decimals)
      const userAssetAccount = getUserAta(poolAccount.data.assetMint)
      return program.methods
        .borrow(amountBN)
        .accounts({
          marketConfig: marketConfigPda,
          assetPool: poolAddress,
          userPosition: userPositionPda,
          user: wallet.publicKey,
          userAssetAccount,
          assetVault: poolAccount.data.assetVault,
          assetMint: poolAccount.data.assetMint,
          tokenProgram: TOKEN_PROGRAM_ID,
          pythPriceFeedAccount: poolAccount.data.pythPriceFeed,
          chainlinkPriceFeedAccount: SystemProgram.programId,
        })
        .rpc()
    },
    onSuccess,
    onError,
  })

  return {
    poolAccount,
    mintDecimalsQuery,
    userPositionAccount,
    createUserPosition,
    depositMutation,
    withdrawMutation,
    borrowMutation,
    repayMutation,
  }
}

// --- Hook for fetching a user's specific token balance ---
export function useUserBalance({ mint }: { mint: PublicKey | undefined | null }) {
  const { connection } = useConnection()
  const { publicKey } = useWallet()

  const ata = useMemo(() => {
    if (!publicKey || !mint) return null
    return getAssociatedTokenAddressSync(mint, publicKey)
  }, [publicKey, mint])

  const query = useQuery({
    queryKey: ['user-balance', { wallet: publicKey?.toBase58(), mint: mint?.toBase58() }],
    queryFn: async () => {
      if (!ata) return null
      try {
        const account = await getAccount(connection, ata)
        return account
      } catch (error) {
        if (error instanceof Error && error.name === 'TokenAccountNotFoundError') {
          return null // ATA doesn't exist, so balance is 0
        }
        throw error
      }
    },
    enabled: !!publicKey && !!mint && !!ata,
  })

  return {
    balance: query.data?.amount,
    isLoading: query.isLoading,
  }
}
