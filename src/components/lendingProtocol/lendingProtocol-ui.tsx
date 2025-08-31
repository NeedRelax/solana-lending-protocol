'use client'

import { PublicKey } from '@solana/web3.js'
import { useLendingProgram, useLendingPool, useUserBalance } from './lendingProtocol-data-access'
import { Button } from '@/components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Input } from '@/components/ui/input'
import { useState } from 'react'
import { ellipsify } from '@/lib/utils'
import { ExplorerLink } from '../cluster/cluster-ui'

// --- Governance UI ---
export function GovernanceCard() {
  const { marketConfigAccount, initializeMarket, addAssetPoolMutation } = useLendingProgram()

  // These addresses should be populated by your `initialize.ts` script
  // !! Remember to update these after running the script !!
  const LOCAL_USDC_MINT = new PublicKey('FHTN1kJ9AmKweupAht4cdZcSxSvCH3FQ3zTnPx9S9EzR')
  const LOCAL_PYTH_FEED = new PublicKey('7ejyKBHUo17btABjnWe3WrEstti944XGQbbqpyeB7aje')

  if (marketConfigAccount.isLoading) {
    return <div className="text-center">Loading Governance Info...</div>
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Governance</CardTitle>
        <CardDescription>Manage the lending protocol on Localnet.</CardDescription>
      </CardHeader>
      <CardContent className="flex gap-4">
        {!marketConfigAccount.data ? (
          <Button onClick={() => initializeMarket.mutate()} disabled={initializeMarket.isPending}>
            Initialize Market {initializeMarket.isPending && '...'}
          </Button>
        ) : (
          <Button
            onClick={() => {
              addAssetPoolMutation.mutate({
                assetMint: LOCAL_USDC_MINT,
                pythPriceFeed: LOCAL_PYTH_FEED,
              })
            }}
            disabled={addAssetPoolMutation.isPending}
          >
            Add Local USDC Pool {addAssetPoolMutation.isPending && '...'}
          </Button>
        )}
      </CardContent>
    </Card>
  )
}

// --- Asset Pools UI ---
export function AssetPoolList() {
  const { assetPools, marketConfigAccount } = useLendingProgram()

  if (assetPools.isLoading) {
    return (
      <div className="text-center">
        <span className="loading loading-spinner loading-lg"></span>
      </div>
    )
  }
  if (!marketConfigAccount.data || marketConfigAccount.data.poolCount === 0) {
    return (
      <div className="text-center p-8 border-dashed border-2 rounded-lg">
        <h2 className="text-2xl font-bold">No Asset Pools Found</h2>
        <p className="mt-2">Use the governance panel to add a new asset pool.</p>
      </div>
    )
  }

  const poolAddresses = marketConfigAccount.data.pools.slice(0, marketConfigAccount.data.poolCount)

  return (
    <div className="space-y-6">
      <h2 className="text-3xl font-bold text-center">Available Asset Pools</h2>
      <div className="grid md:grid-cols-2 gap-6">
        {poolAddresses.map((poolAddress) => (
          <AssetPoolCard key={poolAddress.toBase58()} poolAddress={poolAddress} />
        ))}
      </div>
    </div>
  )
}

function AssetPoolCard({ poolAddress }: { poolAddress: PublicKey }) {
  const {
    poolAccount,
    userPositionAccount,
    mintDecimalsQuery,
    createUserPosition,
    depositMutation,
    withdrawMutation,
    borrowMutation,
    repayMutation,
  } = useLendingPool({ poolAddress })

  const { balance: userTokenBalance, isLoading: isBalanceLoading } = useUserBalance({
    mint: poolAccount.data?.assetMint,
  })

  const [amount, setAmount] = useState('')

  const isLoading =
    poolAccount.isLoading || mintDecimalsQuery.isLoading || userPositionAccount.isLoading || isBalanceLoading

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Loading...</CardTitle>
        </CardHeader>
      </Card>
    )
  }
  if (!poolAccount.data || mintDecimalsQuery.data === undefined) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Pool Data Not Found</CardTitle>
        </CardHeader>
      </Card>
    )
  }

  const decimals = mintDecimalsQuery.data
  const userPosition = userPositionAccount.data

  const collateral = userPosition ? Number(userPosition.collateralAmount) / 10 ** decimals : 0
  const debt = userPosition ? Number(userPosition.loanAmount) / 10 ** decimals : 0
  const formattedUserTokenBalance = userTokenBalance ? Number(userTokenBalance) / 10 ** decimals : 0

  const handleTransaction = (mutation: typeof depositMutation) => {
    if (!amount || isNaN(parseFloat(amount))) return
    mutation.mutate(parseFloat(amount), {
      onSuccess: () => {
        setAmount('') // Reset input on success
      },
    })
  }

  return (
    <Card className="flex flex-col">
      <CardHeader>
        <CardTitle>USDC Pool</CardTitle>
        <CardDescription>
          Pool Address: <ExplorerLink path={`account/${poolAddress}`} label={ellipsify(poolAddress.toString())} />
        </CardDescription>
      </CardHeader>
      <CardContent className="flex-grow space-y-4">
        <div className="p-4 bg-gray-100 dark:bg-gray-800 rounded-md">
          <h3 className="font-bold text-lg">Your Wallet</h3>
          <p>USDC Balance: {formattedUserTokenBalance.toFixed(2)}</p>
        </div>

        {userPosition ? (
          <div className="space-y-4">
            <div>
              <h3 className="font-bold">Your Position in this Pool</h3>
              <p>Collateral: {collateral.toFixed(2)} USDC</p>
              <p>Debt: {debt.toFixed(2)} USDC</p>
            </div>
            <div className="space-y-2">
              <Input
                type="text"
                placeholder="Enter amount in USDC"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
              <div className="grid grid-cols-2 gap-2">
                <Button
                  onClick={() => handleTransaction(depositMutation)}
                  disabled={!amount || depositMutation.isPending}
                >
                  Deposit
                </Button>
                <Button
                  onClick={() => handleTransaction(withdrawMutation)}
                  disabled={!amount || withdrawMutation.isPending}
                >
                  Withdraw
                </Button>
                <Button
                  onClick={() => handleTransaction(borrowMutation)}
                  disabled={!amount || borrowMutation.isPending}
                >
                  Borrow
                </Button>
                <Button onClick={() => handleTransaction(repayMutation)} disabled={!amount || repayMutation.isPending}>
                  Repay
                </Button>
              </div>
            </div>
          </div>
        ) : (
          <div className="text-center">
            <p className="mb-4">You don not have a position in this pool yet.</p>
            <Button onClick={() => createUserPosition.mutate()} disabled={createUserPosition.isPending}>
              Create Position {createUserPosition.isPending && '...'}
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}
