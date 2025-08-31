// anchor/scripts/initialize.ts

import * as anchor from '@coral-xyz/anchor'
import { Program, BN, Wallet } from '@coral-xyz/anchor'
import { LendingProtocol } from '../target/types/lending_protocol'
import { MockWriter } from '../target/types/mock_writer'
import lendingProtocolIdl from '../target/idl/lending_protocol.json'
import mockWriterIdl from '../target/idl/mock_writer.json'

import { TOKEN_PROGRAM_ID, createMint, createAssociatedTokenAccount, mintTo } from '@solana/spl-token'
import { Keypair, PublicKey, SystemProgram, Transaction, Connection, clusterApiUrl } from '@solana/web3.js'
import fs from 'fs'

async function main() {
  // --- 核心修复：手动创建 Provider ---
  // 1. 创建一个 Connection
  // Anchor.toml 中 cluster = "localnet", 所以我们连接到本地
  const connection = new Connection('http://127.0.0.1:8899', 'confirmed')

  // 2. 从文件加载钱包
  // Anchor.toml 中 wallet = "~/.config/solana/id.json"
  // 我们需要解析这个路径
  const homeDir = require('os').homedir()
  const walletPath = require('path').join(homeDir, '.config', 'solana', 'id.json')
  const walletKeypair = Keypair.fromSecretKey(Buffer.from(JSON.parse(fs.readFileSync(walletPath, 'utf-8'))))
  const wallet = new Wallet(walletKeypair)

  // 3. 创建 Provider
  const provider = new anchor.AnchorProvider(connection, wallet, anchor.AnchorProvider.defaultOptions())

  // 4. 设置全局 Provider
  anchor.setProvider(provider)

  const lendingProgram = new Program(lendingProtocolIdl as anchor.Idl, provider) as Program<LendingProtocol>
  const mockOracleProgram = new Program(mockWriterIdl as anchor.Idl, provider) as Program<MockWriter>

  const governance = (provider.wallet as anchor.Wallet).payer
  console.log('Governance Authority:', governance.publicKey.toBase58())

  // --- Create Mock USDC Mint ---
  const usdcMintKeypair = Keypair.generate()
  const usdcMint = await createMint(provider.connection, governance, governance.publicKey, null, 6, usdcMintKeypair)
  console.log('Mock USDC Mint Created:', usdcMint.toBase58())

  // --- Create Mock Pyth Account ---
  const usdcPythAccount = Keypair.generate()
  const createPythTx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: governance.publicKey,
      newAccountPubkey: usdcPythAccount.publicKey,
      space: 3312,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(3312),
      programId: mockOracleProgram.programId,
    }),
  )
  await provider.sendAndConfirm(createPythTx, [usdcPythAccount])
  await mockOracleProgram.methods
    .createFakePyth(new BN(1 * 1e8), new BN(0), -8)
    .accounts({ fakePythAccount: usdcPythAccount.publicKey })
    .rpc()
  console.log('Mock Pyth Oracle Created:', usdcPythAccount.publicKey.toBase58())

  // --- Initialize Market (if needed) ---
  const [marketConfigPda] = PublicKey.findProgramAddressSync([Buffer.from('market_config')], lendingProgram.programId)
  try {
    await lendingProgram.account.marketConfig.fetch(marketConfigPda)
    console.log('Market config already initialized.')
  } catch (e) {
    console.log('Initializing market config...')
    await lendingProgram.methods
      .initializeMarketConfig()
      // --- 核心修复：使用 .accountsStrict() 或手动构建交易 ---
      // 我们将手动构建指令
      .instruction()
      .then(async (ix) => {
        const tx = new Transaction().add(ix)
        // 手动为指令设置账户
        ix.keys = [
          { pubkey: marketConfigPda, isSigner: false, isWritable: true },
          { pubkey: governance.publicKey, isSigner: true, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ]
        return provider.sendAndConfirm(tx, [governance])
      })
    console.log('Market config initialized.')
  }

  // --- Add USDC Asset Pool ---
  console.log('Adding USDC asset pool...')
  const assetPoolKeypair = Keypair.generate()
  const [assetVaultPda] = PublicKey.findProgramAddressSync(
    [Buffer.from('asset_vault'), assetPoolKeypair.publicKey.toBuffer()],
    lendingProgram.programId,
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

  // 使用 accountsStrict 强制 Anchor 使用我们提供的对象
  await lendingProgram.methods
    .addAssetPool(params)
    .accountsStrict({
      marketConfig: marketConfigPda,
      assetPool: assetPoolKeypair.publicKey,
      assetVault: assetVaultPda,
      assetMint: usdcMint,
      governanceAuthority: governance.publicKey,
      pythPriceFeedAccount: usdcPythAccount.publicKey,
      chainlinkPriceFeedAccount: null,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([assetPoolKeypair])
    .rpc()
  console.log('USDC Asset Pool Added:', assetPoolKeypair.publicKey.toBase58())
  console.log('\nSetup complete! You can now start your UI.')
  // --- 核心修复：添加发币步骤 ---
  // a. 定义你的浏览器钱包地址
  // !! 请将这个地址替换为你自己的 Phantom 钱包地址 !!
  const userWalletAddress = new PublicKey('5fW4K8SHAQQbWuEjhnHCKhPAVnU3dejKoLj7xYNcp8kB')

  // b. 创建或获取该钱包的关联代币账户 (ATA)
  const userAta = await createAssociatedTokenAccount(
    provider.connection,
    governance, // Payer
    usdcMint,
    userWalletAddress,
  )
  console.log(`Created ATA ${userAta.toBase58()} for user ${userWalletAddress.toBase58()}`)

  // c. 向该 ATA 发送代币
  const mintAmount = new BN(5000 * 1e6) // 发送 5000 个 USDC
  await mintTo(
    provider.connection,
    governance, // Mint Authority
    usdcMint,
    userAta,
    governance.publicKey, // Authority of the mint authority
    mintAmount.toNumber(), // mintTo takes a number or bigint
  )
  console.log(`Minted ${mintAmount.toNumber() / 1e6} USDC to user's ATA.`)
  // --- 修复结束 ---
}

main().then(
  () => process.exit(0),
  (err) => {
    console.error(err)
    process.exit(1)
  },
)
