import { Connection, Keypair, PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } from '@solana/web3.js'
import fs from 'fs'

// 初始化连接和账户
const connection = new Connection('https://api.devnet.solana.com', 'confirmed')
const payer = Keypair.generate()
const programDataAccount = Keypair.generate()
const programKeypair = Keypair.generate()

// 读取程序的二进制数据
const programData = fs.readFileSync('path_to_program.so')

// 创建并部署程序
async function deployProgram() {
  // 创建程序数据账户
  const createAccountIx = SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: programDataAccount.publicKey,
    lamports: await connection.getMinimumBalanceForRentExemption(programData.length),
    space: programData.length,
    programId: new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111'),
  })

  // 加载程序
  const loadProgramIx = new TransactionInstruction({
    keys: [
      { pubkey: programDataAccount.publicKey, isSigner: false, isWritable: true },
      { pubkey: programKeypair.publicKey, isSigner: true, isWritable: true },
    ],
    programId: new PublicKey('BPFLoaderUpgradeab1e11111111111111111111111'),
    data: Buffer.from([0, ...programData]), // 0 表示加载程序的操作码
  })

  // 创建并发送交易
  const tx = new Transaction().add(createAccountIx, loadProgramIx)
  await sendAndConfirmTransaction(connection, tx, [payer, programDataAccount, programKeypair])
  console.log('Program deployed successfully')
}

// 调用部署函数
deployProgram().catch(console.error)
