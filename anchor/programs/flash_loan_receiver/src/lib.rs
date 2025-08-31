use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};

declare_id!("Agxw43dYHrUcCiJPAeTKe4QK4qfgQPoAPTWrwkQepiw7");

#[program]
pub mod flash_loan_receiver {
    use super::*;

    // 这个指令将被我们的借贷协议回调
    pub fn executeOperation(
        ctx: Context<ExecuteOperationContext>,
        amount: u64,
        fee: u64,
    ) -> Result<()> {
        msg!("Flash loan receiver: operation executing.");
        require!(ctx.accounts.user.is_signer, MyError::UserNotSigner);

        // 1. 计算需要偿还的总金额
        let repay_amount = amount.checked_add(fee).unwrap();
        msg!("Repaying amount: {}", repay_amount);

        // 2. 将本金 + 费用转回
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.vault_token_account.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
            mint: ctx.accounts.mint.to_account_info(),
        };
        let cpi_context =
            CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);

        token_interface::transfer_checked(cpi_context, repay_amount, ctx.accounts.mint.decimals)?;

        msg!("Flash loan receiver: Repayment successful.");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ExecuteOperationContext<'info> {
    /// CHECK: This is the authority of the user_token_account and must be a signer.
    /// We manually check `is_signer` in the instruction logic.
    #[account(mut)]
    pub user: AccountInfo<'info>, // 闪电贷的发起者
    #[account(mut)]
    pub user_token_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)]
    pub vault_token_account: InterfaceAccount<'info, TokenAccount>, // 借贷协议的金库
    pub mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}
// 添加一个自定义错误
#[error_code]
pub enum MyError {
    #[msg("The user account must be a signer.")]
    UserNotSigner,
}
