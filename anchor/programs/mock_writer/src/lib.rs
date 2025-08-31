// in programs/mock_writer/src/lib.rs

use anchor_lang::prelude::*;
use bytemuck;
use pyth_sdk_solana::state::{
    AccountType, PriceAccount, PriceInfo, PriceStatus, SolanaPriceAccount, MAGIC, VERSION_2,
};
declare_id!("BN6SmdB35pe6CF8dA7AehZMB3umBGPQK46TcyB9PNBLc");

#[program]
pub mod mock_writer {
    use super::*;

    pub fn create_fake_pyth(
        ctx: Context<CreateFakePyth>,
        price: i64,
        conf: u64,
        expo: i32,
    ) -> Result<()> {
        // --- 只借用一次 ---
        let account_info = &ctx.accounts.fake_pyth_account;
        let mut account_data = account_info.try_borrow_mut_data()?;

        // 构建一个完整的 SolanaPriceAccount 实例
        let mut price_account = SolanaPriceAccount {
            magic: MAGIC,
            ver: VERSION_2,
            atype: AccountType::Price as u32,
            size: std::mem::size_of::<SolanaPriceAccount>() as u32,
            expo,
            timestamp: Clock::get()?.unix_timestamp,
            agg: PriceInfo {
                price,
                conf,
                // --- 核心修复：明确设置状态为 Trading ---
                status: PriceStatus::Trading,
                // ----------------------------------------
                ..Default::default()
            },
            ..Default::default()
        };

        // 使用 bytemuck 将整个结构体序列化为字节
        let bytes_to_write: &[u8] = bytemuck::bytes_of(&price_account);

        // 将序列化后的字节写入账户数据
        let write_len = bytes_to_write.len();

        // 确保我们不会写入超出账户数据的边界
        if account_data.len() >= write_len {
            account_data[0..write_len].copy_from_slice(bytes_to_write);
        } else {
            // 如果账户空间不足，可以返回一个错误
            // 在这种情况下，我们知道空间是足够的(3312)，所以这更像是一个安全检查
            return err!(ProgramError::AccountDataTooSmall);
        }

        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateFakePyth<'info> {
    /// CHECK: done
    #[account(mut)]
    pub fake_pyth_account: AccountInfo<'info>,
}
#[error_code]
pub enum ProgramError {
    #[msg("The provided account data is too small.")]
    AccountDataTooSmall,
}
