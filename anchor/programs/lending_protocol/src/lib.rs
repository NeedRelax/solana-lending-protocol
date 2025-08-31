// --- 1. Imports ---
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke,
};
use anchor_spl::token_interface::{self, Mint, TokenAccount, TokenInterface, TransferChecked};
use bytemuck::{Pod, Zeroable};
use pyth_sdk_solana::state::SolanaPriceAccount;
use pyth_sdk_solana::{Price, PriceFeed};
// Import U192 for high-precision math
use spl_math::uint::U192;
use std::mem::size_of; // <-- Added

// --- 2. Program ID ---
declare_id!("2XsQQ3t5uScXfiwxWGBLNXBSMwoMfEyw9Muc1LwcC7gH");

// --- 3. Constants ---
// PDA Seeds
const VAULT_SEED: &[u8] = b"asset_vault";
const USER_POSITION_SEED: &[u8] = b"user_position";
const MARKET_CONFIG_SEED: &[u8] = b"market_config";
const CREDIT_DELEGATION_SEED: &[u8] = b"credit_delegation";

// Financial Parameters
const LIQUIDATION_BONUS_BPS: u128 = 500; // 5%
const MAX_LIQUIDATION_RATIO_BPS: u128 = 5000; // 50%
const STALE_PRICE_THRESHOLD_SECONDS: u64 = 60;
const SECONDS_IN_YEAR: u128 = 31_536_000;
const BASIS_POINTS_DIVISOR: u128 = 10_000;
// Constant for scaling high-precision math results back to u128
const PRECISION_DIVISOR: u128 = 1_000_000_000_000; // 10^12
                                                   // Oracle Security Parameter
const MAX_CONFIDENCE_INTERVAL_BPS: u64 = 300; // 3%

// --- 4. Program Module ---
#[program]
pub mod lending_protocol {
    use super::*;

    // --- Governance Instructions ---

    /// [Governance] Initializes the central market configuration.
    pub fn initialize_market_config(ctx: Context<InitializeMarketConfig>) -> Result<()> {
        let config = &mut ctx.accounts.market_config.load_init()?;
        config.governance_authority = ctx.accounts.owner.key();
        config.status = 0; // 0 = Active
        emit!(MarketConfigInitialized {
            new_governance_authority: ctx.accounts.owner.key()
        });
        Ok(())
    }

    /// [Governance] Transfers governance authority to a new public key.
    pub fn update_governance_authority(
        ctx: Context<UpdateGovernanceAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        let config = &mut ctx.accounts.market_config.load_mut()?;
        let old_authority = config.governance_authority;
        config.governance_authority = new_authority;
        emit!(GovernanceAuthorityChanged {
            old_authority,
            new_authority,
        });
        Ok(())
    }

    /// [Governance] Pauses the protocol, halting most user interactions.
    pub fn pause_protocol(ctx: Context<UpdateGovernanceAuthority>) -> Result<()> {
        ctx.accounts.market_config.load_mut()?.status = 1; // 1 = Paused
        emit!(ProtocolPaused {});
        Ok(())
    }

    /// [Governance] Unpauses the protocol, resuming normal operations.
    pub fn unpause_protocol(ctx: Context<UpdateGovernanceAuthority>) -> Result<()> {
        ctx.accounts.market_config.load_mut()?.status = 0; // 0 = Active
        emit!(ProtocolUnpaused {});
        Ok(())
    }

    /// [Governance] Sets the protocol to withdraw-only mode.
    pub fn enable_withdraw_only_mode(ctx: Context<UpdateGovernanceAuthority>) -> Result<()> {
        ctx.accounts.market_config.load_mut()?.status = 2; // 2 = WithdrawOnly
        emit!(ProtocolWithdrawOnlyModeEnabled {});
        Ok(())
    }

    /// [Governance] Adds a new asset pool to the market.
    pub fn add_asset_pool(ctx: Context<AddAssetPool>, params: AssetPoolParams) -> Result<()> {
        let asset_pool_key = ctx.accounts.asset_pool.key();
        let pool = &mut ctx.accounts.asset_pool.load_init()?;

        pool.asset_mint = ctx.accounts.asset_mint.key();
        pool.asset_vault = ctx.accounts.asset_vault.key();
        pool.pyth_price_feed = ctx.accounts.pyth_price_feed_account.key();

        pool.chainlink_price_feed = match ctx.accounts.chainlink_price_feed_account.as_ref() {
            Some(account) => account.key(),
            None => anchor_lang::system_program::ID, // 如果没有提供，就存入哨兵值
        };

        pool.apply_params(params)?;
        pool.last_interest_update_timestamp = Clock::get()?.unix_timestamp;

        ctx.accounts
            .market_config
            .load_mut()?
            .add_pool(asset_pool_key)?;

        emit!(AssetPoolAdded {
            pool_key: asset_pool_key,
            asset_mint: pool.asset_mint,
        });
        Ok(())
    }

    /// [Governance] Updates the parameters for an existing asset pool.
    pub fn update_asset_pool(ctx: Context<UpdateAssetPool>, params: AssetPoolParams) -> Result<()> {
        ctx.accounts.asset_pool.load_mut()?.apply_params(params)?;
        emit!(AssetPoolUpdated {
            pool_key: ctx.accounts.asset_pool.key()
        });
        Ok(())
    }

    /// [Governance] Collects accrued protocol fees from an asset pool.
    pub fn collect_protocol_fees(ctx: Context<CollectProtocolFees>) -> Result<()> {
        let pool = &mut ctx.accounts.asset_pool.load_mut()?;
        let fees_to_collect = pool.accrued_protocol_fees;
        require_gt!(fees_to_collect, 0, LendingError::ZeroAmount);

        pool.accrued_protocol_fees = 0;

        let pool_key = ctx.accounts.asset_pool.key();
        let seeds = &[VAULT_SEED, pool_key.as_ref(), &[ctx.bumps.asset_vault]];

        cpi_utils::transfer_from_vault_checked(
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.destination_account.to_account_info(),
            &ctx.accounts.asset_mint.to_account_info(),
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &[&seeds[..]],
            fees_to_collect,
            ctx.accounts.asset_mint.decimals,
        )?;

        emit!(ProtocolFeesCollected {
            pool: pool_key,
            recipient: ctx.accounts.destination_account.key(),
            amount: fees_to_collect,
        });
        Ok(())
    }

    // --- Core Lending Instructions ---

    /// [User] Initializes a user's position account for a specific asset pool.
    pub fn create_user_position(ctx: Context<CreateUserPosition>) -> Result<()> {
        let position = &mut ctx.accounts.user_position.load_init()?;
        position.owner = ctx.accounts.user.key();
        position.pool = ctx.accounts.asset_pool.key();
        position.collateral_amount = 0;
        position.loan_amount = 0;
        Ok(())
    }

    /// [User] Deposits assets into a pool to be used as collateral.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        require!(
            ctx.accounts.market_config.load()?.status == 0,
            LendingError::ProtocolNotActive
        );
        require_gt!(amount, 0, LendingError::ZeroAmount);
        ctx.accounts.asset_pool.load_mut()?.accrue_interest()?;

        cpi_utils::transfer_from_user_checked(
            &ctx.accounts.user.to_account_info(),
            &ctx.accounts.user_asset_account.to_account_info(),
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.asset_mint.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            amount,
            ctx.accounts.asset_mint.decimals,
        )?;

        let user_position = &mut ctx.accounts.user_position.load_mut()?;
        user_position.collateral_amount = user_position
            .collateral_amount
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        let pool = &mut ctx.accounts.asset_pool.load_mut()?;
        pool.total_deposits = pool
            .total_deposits
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        emit!(Deposited {
            pool: ctx.accounts.asset_pool.key(),
            user: ctx.accounts.user.key(),
            amount
        });
        Ok(())
    }

    /// [User] Withdraws collateral from a pool, subject to health checks.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let market_config = ctx.accounts.market_config.load()?;
        require!(
            market_config.status == 0 || market_config.status == 2, // Active or WithdrawOnly
            LendingError::ProtocolPaused
        );
        require_gt!(amount, 0, LendingError::ZeroAmount);
        let mut pool = ctx.accounts.asset_pool.load_mut()?;
        pool.accrue_interest()?;

        let user_position = &mut ctx.accounts.user_position.load_mut()?;
        let new_collateral_amount = user_position
            .collateral_amount
            .checked_sub(amount)
            .ok_or(LendingError::InsufficientCollateralAmount)?;

        let price = oracle::get_price(
            &ctx.accounts.pyth_price_feed_account,
            Option::from(ctx.accounts.chainlink_price_feed_account.as_ref()),
        )?;
        require!(
            utils::is_healthy(
                new_collateral_amount,
                user_position.loan_amount,
                price,
                pool.liquidation_threshold_bps
            )?,
            LendingError::PositionWouldBecomeUnhealthy
        );

        let pool_key = ctx.accounts.asset_pool.key();
        let seeds = &[VAULT_SEED, pool_key.as_ref(), &[ctx.bumps.asset_vault]];

        cpi_utils::transfer_from_vault_checked(
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.user_asset_account.to_account_info(),
            &ctx.accounts.asset_mint.to_account_info(),
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &[&seeds[..]],
            amount,
            ctx.accounts.asset_mint.decimals,
        )?;

        user_position.collateral_amount = new_collateral_amount;
        pool.total_deposits = pool
            .total_deposits
            .checked_sub(amount)
            .ok_or(LendingError::MathOverflow)?;

        emit!(Withdrawn {
            pool: ctx.accounts.asset_pool.key(),
            user: ctx.accounts.user.key(),
            amount
        });
        Ok(())
    }

    /// [User] Borrows assets against their deposited collateral.
    // in lending-protocol/src/lib.rs

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()> {
        // We load mutable versions at the beginning since we'll need them for updates later.
        // This also performs an implicit "account exists" check.
        let mut pool = ctx.accounts.asset_pool.load_mut()?;
        let mut user_position = ctx.accounts.user_position.load_mut()?;
        let market_config = ctx.accounts.market_config.load()?;

        // --- 1. Manual Account Validation ---
        // This section replaces all the `.load()?` constraints from the Accounts struct.
        msg!("--- Manual Validation Phase ---");

        // a. Validate asset_pool accounts
        require_keys_eq!(
            ctx.accounts.asset_mint.key(),
            pool.asset_mint,
            LendingError::LoanMintMismatch
        ); // Using a more specific error
        require_keys_eq!(
            ctx.accounts.asset_vault.key(),
            pool.asset_vault,
            LendingError::InvalidAssetVault // <-- 使用新的、语义正确的错误码
        );

        // b. Validate user_position owner
        require_keys_eq!(
            user_position.owner,
            ctx.accounts.user.key(),
            LendingError::InvalidOwner
        );

        // c. Validate user_asset_account mint
        require_keys_eq!(
            ctx.accounts.user_asset_account.mint,
            pool.asset_mint,
            LendingError::InvalidAssetVault
        );

        // d. Validate oracle accounts
        require_keys_eq!(
            ctx.accounts.pyth_price_feed_account.key(),
            pool.pyth_price_feed,
            LendingError::InvalidOracleAccount
        );
        require_keys_eq!(
            ctx.accounts.chainlink_price_feed_account.key(),
            pool.chainlink_price_feed,
            LendingError::InvalidOracleAccount
        );

        msg!("--- Manual Validation Passed ---");

        // --- 2. Instruction Logic with Detailed Logging ---
        msg!("--- Entering 'borrow' instruction logic ---");

        // a. Check protocol status
        require!(market_config.status == 0, LendingError::ProtocolNotActive);
        msg!("Step 2.a: Protocol status check passed (status is active).");

        // b. Check amount
        require_gt!(amount, 0, LendingError::ZeroAmount);
        msg!("Step 2.b: Amount check passed (amount is greater than zero).");

        // c. Accrue interest (we already have a mutable `pool`)
        match pool.accrue_interest() {
            Ok(_) => msg!("Step 2.c: Accrue interest completed successfully."),
            Err(e) => {
                msg!("Error: pool.accrue_interest() failed. Reason: {:?}", e);
                return Err(e);
            }
        }

        // d. Calculate new loan amount (we already have a mutable `user_position`)
        let new_loan_amount = match user_position.loan_amount.checked_add(amount) {
            Some(val) => val,
            None => {
                msg!("Error: MathOverflow during new_loan_amount calculation.");
                return err!(LendingError::MathOverflow);
            }
        };
        msg!(
            "Step 2.d: New loan amount calculated successfully: {}",
            new_loan_amount
        );

        // e. Get oracle price
        msg!("Step 2.e: About to call oracle::get_price.");
        let price = match oracle::get_price(
            &ctx.accounts.pyth_price_feed_account,
            Option::from(ctx.accounts.chainlink_price_feed_account.as_ref()), // Pass as Option<&AccountInfo>
        ) {
            Ok(p) => p,
            Err(e) => {
                msg!(
                    "Error: oracle::get_price returned an error. Reason: {:?}",
                    e
                );
                // This is the most likely place for AllOraclesFailed
                return Err(e);
            }
        };
        msg!(
            "Step 2.e: oracle::get_price executed successfully. Price: {}",
            price.price
        );

        // f. Check eligibility for borrow
        msg!("Step 2.f: About to check is_eligible_for_borrow.");
        require!(
            utils::is_eligible_for_borrow(
                user_position.collateral_amount,
                new_loan_amount,
                price,
                pool.loan_to_value_bps
            )?,
            LendingError::InsufficientCollateral
        );
        msg!("Step 2.f: Eligibility check passed.");

        // --- 3. CPI and State Updates ---
        msg!("--- Performing CPI transfer and state updates ---");

        let pool_key = ctx.accounts.asset_pool.key();
        let seeds = &[VAULT_SEED, pool_key.as_ref(), &[ctx.bumps.asset_vault]];

        cpi_utils::transfer_from_vault_checked(
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.user_asset_account.to_account_info(),
            &ctx.accounts.asset_mint.to_account_info(),
            // Authority for the vault PDA is the vault PDA itself
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &[&seeds[..]],
            amount,
            ctx.accounts.asset_mint.decimals,
        )?;
        msg!("CPI transfer successful.");

        user_position.loan_amount = new_loan_amount;
        pool.total_loans = pool
            .total_loans
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;
        msg!("State updated successfully.");

        emit!(Borrowed {
            pool: ctx.accounts.asset_pool.key(),
            user: ctx.accounts.user.key(),
            amount
        });

        msg!("--- 'borrow' instruction finished successfully ---");
        Ok(())
    }
    /// [User] Repays a loan to the asset pool.
    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()> {
        let market_config = ctx.accounts.market_config.load()?;
        require!(
            market_config.status == 0 || market_config.status == 2, // Active or WithdrawOnly
            LendingError::ProtocolPaused
        );
        require_gt!(amount, 0, LendingError::ZeroAmount);
        ctx.accounts.asset_pool.load_mut()?.accrue_interest()?;

        let user_position = &mut ctx.accounts.user_position.load_mut()?;
        let actual_repayment = amount.min(user_position.loan_amount);

        if actual_repayment == 0 {
            return Ok(());
        }

        cpi_utils::transfer_from_user_checked(
            &ctx.accounts.user.to_account_info(),
            &ctx.accounts.user_asset_account.to_account_info(),
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.asset_mint.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            actual_repayment,
            ctx.accounts.asset_mint.decimals,
        )?;

        user_position.loan_amount = user_position
            .loan_amount
            .checked_sub(actual_repayment)
            .ok_or(LendingError::MathOverflow)?;

        let pool = &mut ctx.accounts.asset_pool.load_mut()?;
        pool.total_loans = pool
            .total_loans
            .checked_sub(actual_repayment)
            .ok_or(LendingError::MathOverflow)?;

        emit!(Repaid {
            pool: ctx.accounts.asset_pool.key(),
            user: ctx.accounts.user.key(),
            amount: actual_repayment
        });
        Ok(())
    }

    /// [Liquidator] Liquidates an unhealthy position by repaying debt to seize collateral.
    pub fn liquidate(ctx: Context<Liquidate>, amount_to_repay: u64) -> Result<()> {
        // --- 1. Load accounts once at the beginning ---
        let collateral_pool = ctx.accounts.collateral_pool.load()?;
        let loan_pool = ctx.accounts.loan_pool.load()?;
        let borrower_collateral_pos = ctx.accounts.borrower_collateral_position.load()?;
        let borrower_loan_pos = ctx.accounts.borrower_loan_position.load()?;
        let market_config = ctx.accounts.market_config.load()?;

        // --- 2. Manual Account Validation ---
        // This replaces all constraints removed from the Accounts struct
        require_keys_eq!(
            ctx.accounts.collateral_mint.key(),
            collateral_pool.asset_mint,
            LendingError::CollateralMintMismatch
        );
        require_keys_eq!(
            ctx.accounts.loan_mint.key(),
            loan_pool.asset_mint,
            LendingError::LoanMintMismatch
        );
        require_keys_eq!(
            borrower_collateral_pos.owner,
            ctx.accounts.borrower.key(),
            LendingError::InvalidOwner
        );
        require_keys_eq!(
            borrower_loan_pos.owner,
            ctx.accounts.borrower.key(),
            LendingError::InvalidOwner
        );
        require_keys_eq!(
            ctx.accounts.liquidator_collateral_account.mint,
            collateral_pool.asset_mint,
            LendingError::CollateralMintMismatch
        );
        require_keys_eq!(
            ctx.accounts.liquidator_loan_account.mint,
            loan_pool.asset_mint,
            LendingError::LoanMintMismatch
        );
        require_keys_eq!(
            ctx.accounts.collateral_vault.key(),
            collateral_pool.asset_vault,
            LendingError::InvalidAssetVault
        );
        require_keys_eq!(
            ctx.accounts.loan_vault.key(),
            loan_pool.asset_vault,
            LendingError::InvalidAssetVault
        );
        require_keys_eq!(
            ctx.accounts.collateral_price_feed_account.key(),
            collateral_pool.pyth_price_feed,
            LendingError::InvalidOracleAccount
        );
        require_keys_eq!(
            ctx.accounts.collateral_chainlink_feed_account.key(),
            collateral_pool.chainlink_price_feed,
            LendingError::InvalidOracleAccount
        );
        require_keys_eq!(
            ctx.accounts.loan_price_feed_account.key(),
            loan_pool.pyth_price_feed,
            LendingError::InvalidOracleAccount
        );
        require_keys_eq!(
            ctx.accounts.loan_chainlink_feed_account.key(),
            loan_pool.chainlink_price_feed,
            LendingError::InvalidOracleAccount
        );

        // --- 3. Instruction Logic ---
        require!(market_config.status == 0, LendingError::ProtocolNotActive);
        require_keys_neq!(
            ctx.accounts.liquidator.key(),
            ctx.accounts.borrower.key(),
            LendingError::CannotLiquidateSelf
        );

        // Now load mutable versions for updates
        let mut collateral_pool_mut = ctx.accounts.collateral_pool.load_mut()?;
        let mut loan_pool_mut = ctx.accounts.loan_pool.load_mut()?;
        collateral_pool_mut.accrue_interest()?;
        loan_pool_mut.accrue_interest()?;

        require!(
            ctx.accounts.is_liquidatable()?,
            LendingError::PositionHealthy
        );

        // Reloading is necessary because accrue_interest modifies them
        let borrower_loan_amount = ctx.accounts.borrower_loan_position.load()?.loan_amount;
        let borrower_collateral_amount = ctx
            .accounts
            .borrower_collateral_position
            .load()?
            .collateral_amount;

        let max_repay_amount = U192::from(borrower_loan_amount)
            .checked_mul(U192::from(MAX_LIQUIDATION_RATIO_BPS))
            .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
            .map(|v| v.as_u64())
            .ok_or(LendingError::MathOverflow)?;
        let actual_repay_amount = amount_to_repay
            .min(max_repay_amount)
            .min(borrower_loan_amount);
        require_gt!(actual_repay_amount, 0, LendingError::ZeroAmount);

        let collateral_to_liquidator_amount = ctx
            .accounts
            .calculate_liquidation_bonus(actual_repay_amount)?;
        require!(
            collateral_to_liquidator_amount <= borrower_collateral_amount,
            LendingError::InsufficientCollateralForLiquidation
        );

        cpi_utils::transfer_from_user_checked(
            &ctx.accounts.liquidator.to_account_info(),
            &ctx.accounts.liquidator_loan_account.to_account_info(),
            &ctx.accounts.loan_vault.to_account_info(),
            &ctx.accounts.loan_mint.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            actual_repay_amount,
            ctx.accounts.loan_mint.decimals,
        )?;

        let collateral_pool_key = ctx.accounts.collateral_pool.key();
        let seeds = &[
            VAULT_SEED,
            collateral_pool_key.as_ref(),
            &[ctx.bumps.collateral_vault],
        ];
        cpi_utils::transfer_from_vault_checked(
            &ctx.accounts.collateral_vault.to_account_info(),
            &ctx.accounts.liquidator_collateral_account.to_account_info(),
            &ctx.accounts.collateral_mint.to_account_info(),
            &ctx.accounts.collateral_vault.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &[&seeds[..]],
            collateral_to_liquidator_amount,
            ctx.accounts.collateral_mint.decimals,
        )?;

        let mut borrower_loan_position = ctx.accounts.borrower_loan_position.load_mut()?;
        borrower_loan_position.loan_amount = borrower_loan_position
            .loan_amount
            .checked_sub(actual_repay_amount)
            .ok_or(LendingError::MathOverflow)?;

        let mut borrower_collateral_position =
            ctx.accounts.borrower_collateral_position.load_mut()?;
        borrower_collateral_position.collateral_amount = borrower_collateral_position
            .collateral_amount
            .checked_sub(collateral_to_liquidator_amount)
            .ok_or(LendingError::MathOverflow)?;

        loan_pool_mut.total_loans = loan_pool_mut
            .total_loans
            .checked_sub(actual_repay_amount)
            .ok_or(LendingError::MathOverflow)?;
        collateral_pool_mut.total_deposits = collateral_pool_mut
            .total_deposits
            .checked_sub(collateral_to_liquidator_amount)
            .ok_or(LendingError::MathOverflow)?;

        emit!(Liquidation {
            collateral_pool: ctx.accounts.collateral_pool.key(),
            loan_pool: ctx.accounts.loan_pool.key(),
            liquidator: ctx.accounts.liquidator.key(),
            borrower: ctx.accounts.borrower.key(),
            repay_amount: actual_repay_amount,
            seized_collateral_amount: collateral_to_liquidator_amount,
        });
        Ok(())
    }

    /// [User/Bot] Executes a flash loan.
    pub fn flash_loan(
        ctx: Context<FlashLoan>,
        amount: u64,
        callback_ix_data: Vec<u8>,
    ) -> Result<()> {
        require!(
            ctx.accounts.market_config.load()?.status == 0,
            LendingError::ProtocolNotActive
        );
        let mut pool = ctx.accounts.asset_pool.load_mut()?;
        require_gt!(amount, 0, LendingError::ZeroAmount);
        require_gt!(
            pool.flash_loan_fee_bps,
            0,
            LendingError::FlashLoanNotAvailable
        );

        let receiver_program_id = ctx.accounts.flash_loan_receiver_program.key();
        require_keys_neq!(
            receiver_program_id,
            *ctx.program_id,
            LendingError::FlashLoanReentrancy
        );

        let fee = U192::from(amount)
            .checked_mul(U192::from(pool.flash_loan_fee_bps))
            .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
            .map(|v| v.as_u64())
            .ok_or(LendingError::MathOverflow)?;
        let vault_balance_before = ctx.accounts.asset_vault.amount;
        require!(
            vault_balance_before >= amount,
            LendingError::InsufficientLiquidity
        );

        let pool_key = ctx.accounts.asset_pool.key();
        let seeds = &[VAULT_SEED, pool_key.as_ref(), &[ctx.bumps.asset_vault]];
        cpi_utils::transfer_from_vault_checked(
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.destination_account.to_account_info(),
            &ctx.accounts.asset_mint.to_account_info(),
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &[&seeds[..]],
            amount,
            ctx.accounts.asset_mint.decimals,
        )?;

        let instruction = Instruction {
            program_id: receiver_program_id,
            accounts: ctx
                .remaining_accounts
                .iter()
                .map(|acc| {
                    if acc.is_writable {
                        AccountMeta::new(acc.key(), acc.is_signer)
                    } else {
                        AccountMeta::new_readonly(acc.key(), acc.is_signer)
                    }
                })
                .collect(),
            data: callback_ix_data,
        };
        invoke(&instruction, ctx.remaining_accounts)?;

        ctx.accounts.asset_vault.reload()?;
        let vault_balance_after = ctx.accounts.asset_vault.amount;
        let expected_balance_after = vault_balance_before
            .checked_add(fee)
            .ok_or(LendingError::MathOverflow)?;
        require!(
            vault_balance_after >= expected_balance_after,
            LendingError::FlashLoanRepaymentFailed
        );

        let actual_fee_earned = vault_balance_after.saturating_sub(vault_balance_before);
        pool.accrued_protocol_fees = pool
            .accrued_protocol_fees
            .checked_add(actual_fee_earned)
            .ok_or(LendingError::MathOverflow)?;
        pool.total_deposits = pool
            .total_deposits
            .checked_add(actual_fee_earned)
            .ok_or(LendingError::MathOverflow)?;

        emit!(FlashLoaned {
            pool: pool_key,
            receiver: ctx.accounts.destination_account.owner,
            amount,
            fee: actual_fee_earned
        });
        Ok(())
    }

    /// [User] Approves another account to borrow against their position.
    pub fn approve_delegation(ctx: Context<ApproveDelegationAccounts>, amount: u64) -> Result<()> {
        msg!("--- Entering 'approve_delegation' instruction ---");

        let mut delegation = match ctx.accounts.credit_delegation.load_init() {
            Ok(d) => {
                msg!("Step 1: credit_delegation.load_init() succeeded.");
                d
            }
            Err(e) => {
                msg!(
                    "Error: credit_delegation.load_init() FAILED. Reason: {:?}",
                    e
                );
                return Err(e);
            }
        };

        msg!("Step 2: Assigning owner.");
        delegation.owner = ctx.accounts.owner.key();

        msg!("Step 3: Assigning delegatee.");
        delegation.delegatee = ctx.accounts.delegatee_account.key();

        msg!("Step 4: Assigning asset pool.");
        delegation.asset_pool = ctx.accounts.asset_pool.key();

        msg!("Step 5: Assigning amounts.");
        delegation.delegated_amount = amount;
        delegation.initial_delegated_amount = amount;

        emit!(DelegationUpdated {
            owner: delegation.owner,
            delegatee: delegation.delegatee,
            pool: delegation.asset_pool,
            delegated_amount: amount
        });

        msg!("--- 'approve_delegation' instruction finished successfully ---");
        Ok(())
    }

    /// [User] Revokes an existing credit delegation.
    pub fn revoke_delegation(ctx: Context<RevokeDelegationAccounts>) -> Result<()> {
        let delegation = &ctx.accounts.credit_delegation.load()?;
        emit!(DelegationUpdated {
            owner: delegation.owner,
            delegatee: delegation.delegatee,
            pool: delegation.asset_pool,
            delegated_amount: 0
        });
        Ok(())
    }

    /// [Delegatee] Borrows using the credit line delegated to them.
    pub fn borrow_delegated(ctx: Context<BorrowDelegated>, amount: u64) -> Result<()> {
        // --- 1. Load all accounts mutably at the beginning ---
        let market_config = ctx.accounts.market_config.load()?;
        let mut pool = ctx.accounts.asset_pool.load_mut()?;
        let mut owner_position = ctx.accounts.owner_position.load_mut()?;
        let mut credit_delegation = ctx.accounts.credit_delegation.load_mut()?;

        // --- 2. Manual Account Validation ---
        msg!("--- borrow_delegated: Manual Validation Phase ---");

        // a. Validate owner_position
        require_keys_eq!(
            owner_position.owner,
            ctx.accounts.owner.key(),
            LendingError::InvalidOwner
        );
        require_keys_eq!(
            owner_position.pool,
            ctx.accounts.asset_pool.key(),
            LendingError::InvalidAssetPool
        );

        // b. Validate credit_delegation
        require_keys_eq!(
            credit_delegation.owner,
            ctx.accounts.owner.key(),
            LendingError::InvalidOwner
        );
        require_keys_eq!(
            credit_delegation.delegatee,
            ctx.accounts.delegatee.key(),
            LendingError::DelegationMismatch
        );

        // c. Validate delegatee_token_account
        require_keys_eq!(
            ctx.accounts.delegatee_token_account.mint,
            pool.asset_mint,
            LendingError::InvalidAssetMint,
        );

        // d. Validate oracle accounts
        require_keys_eq!(
            ctx.accounts.pyth_price_feed_account.key(),
            pool.pyth_price_feed,
            LendingError::InvalidOracleAccount
        );
        require_keys_eq!(
            ctx.accounts.chainlink_price_feed_account.key(),
            pool.chainlink_price_feed,
            LendingError::InvalidOracleAccount
        );

        msg!("--- borrow_delegated: Manual Validation Passed ---");

        // --- 3. Instruction Logic ---
        require!(market_config.status == 0, LendingError::ProtocolNotActive);
        require_gt!(amount, 0, LendingError::ZeroAmount);
        require!(
            amount <= credit_delegation.delegated_amount,
            LendingError::DelegationExceeded
        );

        pool.accrue_interest()?;

        let new_loan_amount = owner_position
            .loan_amount
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        let price = oracle::get_price(
            &ctx.accounts.pyth_price_feed_account,
            Option::from(ctx.accounts.chainlink_price_feed_account.as_ref()),
        )?;

        require!(
            utils::is_eligible_for_borrow(
                owner_position.collateral_amount,
                new_loan_amount,
                price,
                pool.loan_to_value_bps
            )?,
            LendingError::InsufficientCollateral
        );

        let pool_key = ctx.accounts.asset_pool.key();
        let seeds = &[VAULT_SEED, pool_key.as_ref(), &[ctx.bumps.asset_vault]];

        cpi_utils::transfer_from_vault_checked(
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.delegatee_token_account.to_account_info(),
            &ctx.accounts.asset_mint.to_account_info(),
            &ctx.accounts.asset_vault.to_account_info(),
            &ctx.accounts.token_program.to_account_info(),
            &[&seeds[..]],
            amount,
            ctx.accounts.asset_mint.decimals,
        )?;

        credit_delegation.delegated_amount = credit_delegation
            .delegated_amount
            .checked_sub(amount)
            .ok_or(LendingError::MathOverflow)?;
        owner_position.loan_amount = new_loan_amount;
        pool.total_loans = pool
            .total_loans
            .checked_add(amount)
            .ok_or(LendingError::MathOverflow)?;

        emit!(BorrowedDelegated {
            pool: ctx.accounts.asset_pool.key(),
            owner: ctx.accounts.owner.key(),
            delegatee: ctx.accounts.delegatee.key(),
            amount
        });

        Ok(())
    }
    /// [User] Executes a batch of operations in a single transaction.
    pub fn execute_operations(
        ctx: Context<ExecuteOperations>,
        operations: Vec<Operation>,
    ) -> Result<()> {
        require!(
            ctx.accounts.market_config.load()?.status == 0,
            LendingError::ProtocolNotActive
        );
        let mut pool = ctx.accounts.asset_pool.load_mut()?;
        let mut position = ctx.accounts.user_position.load_mut()?;
        pool.accrue_interest()?;

        let mut current_collateral = position.collateral_amount;
        let mut current_debt = position.loan_amount;
        let pool_key = ctx.accounts.asset_pool.key();
        let seeds = &[VAULT_SEED, pool_key.as_ref(), &[ctx.bumps.asset_vault]];

        let asset_mint_info = ctx.accounts.asset_mint.to_account_info();
        let asset_decimals = ctx.accounts.asset_mint.decimals;
        let price = oracle::get_price(
            &ctx.accounts.pyth_price_feed_account,
            Option::from(&ctx.accounts.chainlink_price_feed_account),
        )?;

        for op in operations {
            match op {
                Operation::Deposit { amount } => {
                    require_gt!(amount, 0, LendingError::InvalidOperation);
                    cpi_utils::transfer_from_user_checked(
                        &ctx.accounts.user.to_account_info(),
                        &ctx.accounts.user_asset_account.to_account_info(),
                        &ctx.accounts.asset_vault.to_account_info(),
                        &asset_mint_info,
                        &ctx.accounts.token_program.to_account_info(),
                        amount,
                        asset_decimals,
                    )?;
                    current_collateral = current_collateral
                        .checked_add(amount)
                        .ok_or(LendingError::MathOverflow)?;
                    pool.total_deposits = pool
                        .total_deposits
                        .checked_add(amount)
                        .ok_or(LendingError::MathOverflow)?;
                }
                Operation::Withdraw { amount } => {
                    require_gt!(amount, 0, LendingError::InvalidOperation);
                    let new_collateral = current_collateral
                        .checked_sub(amount)
                        .ok_or(LendingError::InsufficientCollateralAmount)?;
                    require!(
                        utils::is_healthy(
                            new_collateral,
                            current_debt,
                            price,
                            pool.liquidation_threshold_bps
                        )?,
                        LendingError::PositionWouldBecomeUnhealthy
                    );
                    cpi_utils::transfer_from_vault_checked(
                        &ctx.accounts.asset_vault.to_account_info(),
                        &ctx.accounts.user_asset_account.to_account_info(),
                        &asset_mint_info,
                        &ctx.accounts.asset_vault.to_account_info(),
                        &ctx.accounts.token_program.to_account_info(),
                        &[&seeds[..]],
                        amount,
                        asset_decimals,
                    )?;
                    current_collateral = new_collateral;
                    pool.total_deposits = pool
                        .total_deposits
                        .checked_sub(amount)
                        .ok_or(LendingError::MathOverflow)?;
                }
                Operation::Borrow { amount } => {
                    require_gt!(amount, 0, LendingError::InvalidOperation);
                    let new_debt = current_debt
                        .checked_add(amount)
                        .ok_or(LendingError::MathOverflow)?;
                    require!(
                        utils::is_eligible_for_borrow(
                            current_collateral,
                            new_debt,
                            price,
                            pool.loan_to_value_bps
                        )?,
                        LendingError::InsufficientCollateral
                    );
                    cpi_utils::transfer_from_vault_checked(
                        &ctx.accounts.asset_vault.to_account_info(),
                        &ctx.accounts.user_asset_account.to_account_info(),
                        &asset_mint_info,
                        &ctx.accounts.asset_vault.to_account_info(),
                        &ctx.accounts.token_program.to_account_info(),
                        &[&seeds[..]],
                        amount,
                        asset_decimals,
                    )?;
                    current_debt = new_debt;
                    pool.total_loans = pool
                        .total_loans
                        .checked_add(amount)
                        .ok_or(LendingError::MathOverflow)?;
                }
                Operation::Repay { amount } => {
                    require_gt!(amount, 0, LendingError::InvalidOperation);
                    let actual_repayment = amount.min(current_debt);
                    if actual_repayment > 0 {
                        cpi_utils::transfer_from_user_checked(
                            &ctx.accounts.user.to_account_info(),
                            &ctx.accounts.user_asset_account.to_account_info(),
                            &ctx.accounts.asset_vault.to_account_info(),
                            &asset_mint_info,
                            &ctx.accounts.token_program.to_account_info(),
                            actual_repayment,
                            asset_decimals,
                        )?;
                        current_debt = current_debt
                            .checked_sub(actual_repayment)
                            .ok_or(LendingError::MathOverflow)?;
                        pool.total_loans = pool
                            .total_loans
                            .checked_sub(actual_repayment)
                            .ok_or(LendingError::MathOverflow)?;
                    }
                }
            }
        }

        position.collateral_amount = current_collateral;
        position.loan_amount = current_debt;

        emit!(OperationsExecuted {
            pool: pool_key,
            user: ctx.accounts.user.key()
        });
        Ok(())
    }
}

// --- 5. Account Context Definitions ---

#[derive(Accounts)]
pub struct InitializeMarketConfig<'info> {
    #[account(init, payer = owner, space = 8 + size_of::<MarketConfig>(), seeds = [MARKET_CONFIG_SEED], bump
    )]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateGovernanceAuthority<'info> {
    #[account(mut, has_one = governance_authority, seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    pub governance_authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(params: AssetPoolParams)]
pub struct AddAssetPool<'info> {
    #[account(mut, has_one = governance_authority)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(init, payer = governance_authority, space = 8 + size_of::<AssetPool>())]
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(init, payer = governance_authority, seeds = [VAULT_SEED, asset_pool.key().as_ref()], bump, token::mint = asset_mint, token::authority = asset_vault
    )]
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)]
    pub governance_authority: Signer<'info>,
    /// CHECK: Pyth price feed account, not deserialized.
    pub pyth_price_feed_account: AccountInfo<'info>,
    /// CHECK: Optional Chainlink price feed account.
    pub chainlink_price_feed_account: Option<AccountInfo<'info>>,
    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct UpdateAssetPool<'info> {
    #[account(mut, has_one = governance_authority)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut)]
    pub asset_pool: AccountLoader<'info, AssetPool>,
    pub governance_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CollectProtocolFees<'info> {
    #[account(has_one = governance_authority)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut, constraint = asset_vault.key() == asset_pool.load()?.asset_vault, constraint = asset_mint.key() == asset_pool.load()?.asset_mint
    )]
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut, seeds = [VAULT_SEED, asset_pool.key().as_ref()], bump)]
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = destination_account.mint == asset_pool.load()?.asset_mint)]
    pub destination_account: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub governance_authority: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct CreateUserPosition<'info> {
    #[account(init, payer = user, space = 8 + size_of::<UserPosition>(), seeds = [USER_POSITION_SEED, asset_pool.key().as_ref(), user.key().as_ref()], bump
    )]
    pub user_position: AccountLoader<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub asset_pool: AccountLoader<'info, AssetPool>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut, constraint = asset_mint.key() == asset_pool.load()?.asset_mint)]
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut, seeds = [USER_POSITION_SEED, asset_pool.key().as_ref(), user.key().as_ref()], bump, constraint = user_position.load()?.owner == user.key() @ LendingError::InvalidOwner
    )]
    pub user_position: AccountLoader<'info, UserPosition>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, constraint = user_asset_account.mint == asset_pool.load()?.asset_mint)]
    pub user_asset_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = asset_vault.key() == asset_pool.load()?.asset_vault)]
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut)] //, constraint = asset_mint.key() == asset_pool.load()?.asset_mint
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut, seeds = [USER_POSITION_SEED, asset_pool.key().as_ref(), user.key().as_ref()], bump,
    )] // constraint = user_position.load()?.owner == user.key() @ LendingError::InvalidOwner
    pub user_position: AccountLoader<'info, UserPosition>,
    pub user: Signer<'info>,
    #[account(mut)] // constraint = user_asset_account.mint == asset_pool.load()?.asset_mint
    pub user_asset_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,  seeds = [VAULT_SEED, asset_pool.key().as_ref()], bump
    )] //constraint = asset_vault.key() == asset_pool.load()?.asset_vault,
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Pyth price feed, address validated against asset_pool.
    #[account()]
    //constraint = pyth_price_feed_account.key() == asset_pool.load()?.pyth_price_feed @ LendingError::InvalidOracleAccount
    pub pyth_price_feed_account: AccountInfo<'info>,
    /// CHECK: Optional Chainlink price feed.
    #[account()]
    //constraint = chainlink_price_feed_account.key() == asset_pool.load()?.chainlink_price_feed @ LendingError::InvalidOracleAccount
    pub chainlink_price_feed_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut)] //constraint = asset_mint.key() == asset_pool.load()?.asset_mint
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut, seeds = [USER_POSITION_SEED, asset_pool.key().as_ref(), user.key().as_ref()], bump
    )] //constraint = user_position.load()?.owner == user.key() @ LendingError::InvalidOwner
    pub user_position: AccountLoader<'info, UserPosition>,
    pub user: Signer<'info>,
    #[account(mut)] //constraint = user_asset_account.mint == asset_pool.load()?.asset_mint
    pub user_asset_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,seeds = [VAULT_SEED, asset_pool.key().as_ref()], bump
    )] // constraint = asset_vault.key() == asset_pool.load()?.asset_vault,
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Pyth price feed, address validated against asset_pool.
    #[account()]
    //constraint = pyth_price_feed_account.key() == asset_pool.load()?.pyth_price_feed @ LendingError::InvalidOracleAccount
    pub pyth_price_feed_account: AccountInfo<'info>,
    /// CHECK: Optional Chainlink price feed.
    #[account()]
    //constraint = chainlink_price_feed_account.key() == asset_pool.load()?.chainlink_price_feed @ LendingError::InvalidOracleAccount
    pub chainlink_price_feed_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut)] //constraint = asset_mint.key() == asset_pool.load()?.asset_mint
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut,  seeds = [USER_POSITION_SEED, asset_pool.key().as_ref(), user.key().as_ref()], bump
    )] //constraint = user_position.load()?.owner ==  user.key() @ LendingError::InvalidOwner
    pub user_position: AccountLoader<'info, UserPosition>,
    pub user: Signer<'info>,
    #[account(mut)] //constraint = user_asset_account.mint == asset_pool.load()?.asset_mint
    pub user_asset_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)] //constraint = asset_vault.key() == asset_pool.load()?.asset_vault
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
}

#[derive(Accounts)]
pub struct Liquidate<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut)]
    pub collateral_pool: AccountLoader<'info, AssetPool>,
    #[account(mut)]
    pub loan_pool: AccountLoader<'info, AssetPool>,
    #[account()]
    //constraint = collateral_mint.key() == collateral_pool.load()?.asset_mint @ LendingError::CollateralMintMismatch
    pub collateral_mint: InterfaceAccount<'info, Mint>,
    #[account()]
    //constraint = loan_mint.key() == loan_pool.load()?.asset_mint @ LendingError::LoanMintMismatch
    pub loan_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, seeds = [USER_POSITION_SEED, collateral_pool.key().as_ref(), borrower.key().as_ref()], bump,
    )] //constraint = borrower_collateral_position.load()?.owner == borrower.key()
    pub borrower_collateral_position: AccountLoader<'info, UserPosition>,
    #[account(mut, seeds = [USER_POSITION_SEED, loan_pool.key().as_ref(), borrower.key().as_ref()], bump,
    )] //constraint = borrower_loan_position.load()?.owner == borrower.key()
    pub borrower_loan_position: AccountLoader<'info, UserPosition>,
    /// CHECK: Borrower's main account, ownership checked on position accounts.
    pub borrower: AccountInfo<'info>,
    pub liquidator: Signer<'info>,
    #[account(mut)]
    //constraint = liquidator_collateral_account.mint == collateral_pool.load()?.asset_mint
    pub liquidator_collateral_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)] //constraint = liquidator_loan_account.mint == loan_pool.load()?.asset_mint
    pub liquidator_loan_account: InterfaceAccount<'info, TokenAccount>,
    #[account(mut,  seeds = [VAULT_SEED, collateral_pool.key().as_ref()], bump
    )] //constraint = collateral_vault.key() == collateral_pool.load()?.asset_vault,
    pub collateral_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut)] //constraint = loan_vault.key() == loan_pool.load()?.asset_vault
    pub loan_vault: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Price feed address validated against collateral_pool account.
    #[account()]
    //constraint = collateral_price_feed_account.key() == collateral_pool.load()?.pyth_price_feed @ LendingError::InvalidOracleAccount
    pub collateral_price_feed_account: AccountInfo<'info>,
    /// CHECK: Optional Chainlink price feed for collateral.
    #[account()]
    //constraint = collateral_chainlink_feed_account.key() == collateral_pool.load()?.chainlink_price_feed @ LendingError::InvalidOracleAccount
    pub collateral_chainlink_feed_account: AccountInfo<'info>,
    /// CHECK: Price feed address validated against loan_pool account.
    #[account()]
    //constraint = loan_price_feed_account.key() == loan_pool.load()?.pyth_price_feed @ LendingError::InvalidOracleAccount
    pub loan_price_feed_account: AccountInfo<'info>,
    /// CHECK: Optional Chainlink price feed for loan.
    #[account()]
    //constraint = loan_chainlink_feed_account.key() == loan_pool.load()?.chainlink_price_feed @ LendingError::InvalidOracleAccount
    pub loan_chainlink_feed_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct FlashLoan<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut, constraint = asset_mint.key() == asset_pool.load()?.asset_mint)]
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut, constraint = asset_vault.key() == asset_pool.load()?.asset_vault, seeds = [VAULT_SEED, asset_pool.key().as_ref()], bump
    )]
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    #[account(mut, constraint = destination_account.mint == asset_pool.load()?.asset_mint)]
    pub destination_account: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: The program that will receive the flash loan and execute a callback.
    pub flash_loan_receiver_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ApproveDelegationAccounts<'info> {
    #[account(
    init_if_needed,
    payer = owner,
    space = 8 + size_of::<CreditDelegation>(),
    seeds = [CREDIT_DELEGATION_SEED, owner.key().as_ref(), asset_pool.key().as_ref(),delegatee_account.key().as_ref()],bump
    )]
    pub credit_delegation: AccountLoader<'info, CreditDelegation>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: The account being delegated to.
    pub delegatee_account: AccountInfo<'info>,
    // pub asset_pool: AccountLoader<'info, AssetPool>,
    /// CHECK: The delegatee account, used for PDA derivation.
    pub asset_pool: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeDelegationAccounts<'info> {
    #[account(mut, seeds = [CREDIT_DELEGATION_SEED, owner.key().as_ref(), asset_pool.key().as_ref(), delegatee_account.key().as_ref()], bump, has_one = owner @ LendingError::InvalidOwner, close = owner
    )]
    pub credit_delegation: AccountLoader<'info, CreditDelegation>,
    #[account(mut)]
    pub owner: Signer<'info>,
    /// CHECK: The delegatee account, used for PDA derivation.
    pub delegatee_account: AccountInfo<'info>,
    // pub asset_pool: AccountLoader<'info, AssetPool>,
    /// CHECK: The delegatee account, used for PDA derivation.
    pub asset_pool: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct BorrowDelegated<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut)] //constraint = asset_mint.key() == asset_pool.load()?.asset_mint
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut,  seeds = [VAULT_SEED, asset_pool.key().as_ref()], bump
    )] //constraint = asset_vault.key() == asset_pool.load()?.asset_vault,
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    #[account(mut)] //has_one = owner,constraint = owner_position.load()?.pool == asset_pool.key()
    pub owner_position: AccountLoader<'info, UserPosition>,
    /// CHECK: Position owner, not a signer.
    pub owner: AccountInfo<'info>,
    #[account(mut, seeds = [CREDIT_DELEGATION_SEED, owner.key().as_ref(), asset_pool.key().as_ref(), delegatee.key().as_ref()], bump,
    )]
    //has_one = owner, constraint = credit_delegation.load()?.delegatee == delegatee.key() @ LendingError::DelegationMismatch
    pub credit_delegation: AccountLoader<'info, CreditDelegation>,
    pub delegatee: Signer<'info>,
    #[account(mut)] //constraint = delegatee_token_account.mint == asset_pool.load()?.asset_mint
    pub delegatee_token_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Price feed address validated against asset_pool.
    #[account()]
    //constraint = pyth_price_feed_account.key() == asset_pool.load()?.pyth_price_feed @ LendingError::InvalidOracleAccount
    pub pyth_price_feed_account: AccountInfo<'info>,
    /// CHECK: Optional Chainlink price feed.
    #[account()]
    //constraint = chainlink_price_feed_account.key() == asset_pool.load()?.chainlink_price_feed @ LendingError::InvalidOracleAccount
    pub chainlink_price_feed_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ExecuteOperations<'info> {
    #[account(seeds = [MARKET_CONFIG_SEED], bump)]
    pub market_config: AccountLoader<'info, MarketConfig>,
    #[account(mut, constraint = asset_mint.key() == asset_pool.load()?.asset_mint)]
    pub asset_pool: AccountLoader<'info, AssetPool>,
    #[account(mut, constraint = asset_vault.key() == asset_pool.load()?.asset_vault, seeds = [VAULT_SEED, asset_pool.key().as_ref()], bump
    )]
    pub asset_vault: InterfaceAccount<'info, TokenAccount>,
    pub asset_mint: InterfaceAccount<'info, Mint>,
    #[account(mut, constraint = user_position.load()?.owner == user.key() @ LendingError::InvalidOwner, seeds = [USER_POSITION_SEED, asset_pool.key().as_ref(), user.key().as_ref()], bump
    )]
    pub user_position: AccountLoader<'info, UserPosition>,
    pub user: Signer<'info>,
    #[account(mut, constraint = user_asset_account.owner == user.key() && user_asset_account.mint == asset_pool.load()?.asset_mint
    )]
    pub user_asset_account: InterfaceAccount<'info, TokenAccount>,
    pub token_program: Interface<'info, TokenInterface>,
    /// CHECK: Price feed address validated against asset_pool.
    #[account()]
    //constraint = pyth_price_feed_account.key() == asset_pool.load()?.pyth_price_feed @ LendingError::InvalidOracleAccount
    pub pyth_price_feed_account: AccountInfo<'info>,
    /// CHECK: Optional Chainlink price feed.
    #[account(constraint = chainlink_price_feed_account.key() == asset_pool.load()?.chainlink_price_feed @ LendingError::InvalidOracleAccount
    )]
    pub chainlink_price_feed_account: AccountInfo<'info>,
}

// --- 6. Events ---

#[event]
pub struct MarketConfigInitialized {
    pub new_governance_authority: Pubkey,
}
#[event]
pub struct GovernanceAuthorityChanged {
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
}
#[event]
pub struct ProtocolPaused {}
#[event]
pub struct ProtocolUnpaused {}
#[event]
pub struct ProtocolWithdrawOnlyModeEnabled {}
#[event]
pub struct AssetPoolAdded {
    pub pool_key: Pubkey,
    pub asset_mint: Pubkey,
}
#[event]
pub struct AssetPoolUpdated {
    pub pool_key: Pubkey,
}
#[event]
pub struct ProtocolFeesCollected {
    pub pool: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
}
#[event]
pub struct Deposited {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}
#[event]
pub struct Withdrawn {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}
#[event]
pub struct Borrowed {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}
#[event]
pub struct Repaid {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}
#[event]
pub struct Liquidation {
    pub collateral_pool: Pubkey,
    pub loan_pool: Pubkey,
    pub liquidator: Pubkey,
    pub borrower: Pubkey,
    pub repay_amount: u64,
    pub seized_collateral_amount: u64,
}
#[event]
pub struct FlashLoaned {
    pub pool: Pubkey,
    pub receiver: Pubkey,
    pub amount: u64,
    pub fee: u64,
}
#[event]
pub struct DelegationUpdated {
    pub owner: Pubkey,
    pub delegatee: Pubkey,
    pub pool: Pubkey,
    pub delegated_amount: u64,
}
#[event]
pub struct BorrowedDelegated {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub delegatee: Pubkey,
    pub amount: u64,
}
#[event]
pub struct OperationsExecuted {
    pub pool: Pubkey,
    pub user: Pubkey,
}

// --- 7. State Account Definitions ---

#[account(zero_copy)]
#[repr(C, packed)]
#[derive(Default)]
pub struct MarketConfig {
    pub governance_authority: Pubkey,
    /// Protocol status: 0=Active, 1=Paused, 2=WithdrawOnly
    pub status: u8,
    pub pool_count: u16,
    pub pools: [Pubkey; MarketConfig::MAX_POOLS],
}

impl MarketConfig {
    pub const MAX_POOLS: usize = 32;

    pub fn add_pool(&mut self, pool_key: Pubkey) -> Result<()> {
        require!(
            self.pool_count < (Self::MAX_POOLS as u16),
            LendingError::MaxAssetsExceeded
        );
        self.pools[self.pool_count as usize] = pool_key;
        self.pool_count += 1;
        Ok(())
    }
}

#[account(zero_copy)]
#[repr(C, packed)]
#[derive(Default, Debug)]
pub struct AssetPool {
    pub asset_mint: Pubkey,
    pub asset_vault: Pubkey,
    pub pyth_price_feed: Pubkey,
    pub chainlink_price_feed: Pubkey,
    pub total_deposits: u64,
    pub total_loans: u64,
    pub accrued_protocol_fees: u64,
    pub last_interest_update_timestamp: i64,
    pub loan_to_value_bps: u64,
    pub liquidation_threshold_bps: u64,
    pub base_borrow_rate_bps: u128,
    pub base_slope_bps: u128,
    pub optimal_utilization_bps: u64,
    pub kink_slope_bps: u128,
    pub protocol_fee_bps: u64,
    pub flash_loan_fee_bps: u64,
}

#[account(zero_copy)]
#[repr(C)]
#[derive(Default, Debug)]
pub struct UserPosition {
    pub owner: Pubkey,
    pub pool: Pubkey,
    pub collateral_amount: u64,
    pub loan_amount: u64,
}

#[account(zero_copy)]
#[repr(C)]
#[derive(Default)]
pub struct CreditDelegation {
    pub owner: Pubkey,
    pub delegatee: Pubkey,
    pub asset_pool: Pubkey,
    pub initial_delegated_amount: u64,
    pub delegated_amount: u64,
}

// --- 8. Parameters & Enums ---

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AssetPoolParams {
    pub loan_to_value_bps: u64,
    pub liquidation_threshold_bps: u64,
    pub base_borrow_rate_bps: u128,
    pub base_slope_bps: u128,
    pub optimal_utilization_bps: u64,
    pub kink_slope_bps: u128,
    pub protocol_fee_bps: u64,
    pub flash_loan_fee_bps: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum Operation {
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
    Borrow { amount: u64 },
    Repay { amount: u64 },
}

// --- 9. Logic Implementations & Helpers ---

impl<'info> Liquidate<'info> {
    pub fn is_liquidatable(&self) -> Result<bool> {
        let collateral_price = oracle::get_price(
            &self.collateral_price_feed_account,
            Option::from(self.collateral_chainlink_feed_account.as_ref()),
        )?;
        let loan_price = oracle::get_price(
            &self.loan_price_feed_account,
            Option::from(&self.loan_chainlink_feed_account),
        )?;

        let collateral_value = utils::calculate_asset_value(
            self.borrower_collateral_position.load()?.collateral_amount,
            collateral_price,
        )?;
        let loan_value = utils::calculate_asset_value(
            self.borrower_loan_position.load()?.loan_amount,
            loan_price,
        )?;

        if loan_value == 0 {
            return Ok(false);
        }

        let liquidation_threshold = self.collateral_pool.load()?.liquidation_threshold_bps as u128;
        let threshold_value = collateral_value
            .checked_mul(liquidation_threshold)
            .and_then(|v| v.checked_div(BASIS_POINTS_DIVISOR))
            .ok_or(LendingError::MathOverflow)?;

        Ok(loan_value > threshold_value)
    }

    pub fn calculate_liquidation_bonus(&self, repay_amount: u64) -> Result<u64> {
        let loan_price = oracle::get_price(
            &self.loan_price_feed_account,
            Option::from(self.loan_chainlink_feed_account.as_ref()),
        )?;
        let collateral_price = oracle::get_price(
            &self.collateral_price_feed_account,
            Option::from(self.collateral_chainlink_feed_account.as_ref()),
        )?;

        let repay_value = utils::calculate_asset_value(repay_amount, loan_price)?;
        let total_value_to_seize = repay_value
            .checked_mul(
                BASIS_POINTS_DIVISOR
                    .checked_add(LIQUIDATION_BONUS_BPS)
                    .ok_or(LendingError::MathOverflow)?,
            )
            .and_then(|v| v.checked_div(BASIS_POINTS_DIVISOR))
            .ok_or(LendingError::MathOverflow)?;

        utils::calculate_amount_from_value(total_value_to_seize, collateral_price)
    }
}

impl AssetPool {
    /// Applies and validates new parameters for an asset pool.
    pub fn apply_params(&mut self, params: AssetPoolParams) -> Result<()> {
        // --- 新增的调试日志 ---
        msg!("--- Debugging apply_params ---");
        msg!("Received loan_to_value_bps: {}", params.loan_to_value_bps);
        msg!(
            "Received liquidation_threshold_bps: {}",
            params.liquidation_threshold_bps
        );
        msg!(
            "Executing check: params.loan_to_value_bps ({}) <= params.liquidation_threshold_bps ({})",
            params.loan_to_value_bps,
            params.liquidation_threshold_bps
        );
        // --- 调试日志结束 ---

        require!(
            params.loan_to_value_bps <= params.liquidation_threshold_bps,
            LendingError::InvalidLtv
        );
        require!(
            params.liquidation_threshold_bps < BASIS_POINTS_DIVISOR as u64,
            LendingError::InvalidLiquidationThreshold
        );
        require!(
            params.optimal_utilization_bps < BASIS_POINTS_DIVISOR as u64,
            LendingError::InvalidOptimalUtilization
        );

        self.loan_to_value_bps = params.loan_to_value_bps;
        self.liquidation_threshold_bps = params.liquidation_threshold_bps;
        self.base_borrow_rate_bps = params.base_borrow_rate_bps;
        self.base_slope_bps = params.base_slope_bps;
        self.optimal_utilization_bps = params.optimal_utilization_bps;
        self.kink_slope_bps = params.kink_slope_bps;
        self.protocol_fee_bps = params.protocol_fee_bps;
        self.flash_loan_fee_bps = params.flash_loan_fee_bps;
        Ok(())
    }
    /// Accrues interest for the pool based on the current utilization rate.
    pub fn accrue_interest(&mut self) -> Result<()> {
        msg!("--- Entering 'accrue_interest' ---");

        let now = match Clock::get() {
            Ok(clock) => clock.unix_timestamp,
            Err(e) => {
                msg!(
                    "Error in accrue_interest: Failed to get clock. Reason: {:?}",
                    e
                );
                return Err(Error::from(e));
            }
        };
        msg!("accrue_interest: current timestamp: {}", now);

        let time_delta = now.saturating_sub(self.last_interest_update_timestamp) as u128;
        msg!("accrue_interest: time_delta: {}", time_delta);

        if time_delta == 0 || self.total_loans == 0 {
            self.last_interest_update_timestamp = now;
            msg!("accrue_interest: no time delta or no loans. Exiting early.");
            return Ok(());
        }

        let utilization_bps = if self.total_deposits == 0 {
            0
        } else {
            U192::from(self.total_loans)
                .checked_mul(U192::from(BASIS_POINTS_DIVISOR))
                .and_then(|v| v.checked_div(U192::from(self.total_deposits)))
                .map(|v| v.as_u128())
                .unwrap_or(u128::MAX)
        };
        let current_apy_bps = self.calculate_current_apy(utilization_bps)?;
        let total_interest_precise = U192::from(self.total_loans)
            .checked_mul(U192::from(current_apy_bps))
            .and_then(|v| v.checked_mul(U192::from(time_delta)))
            .and_then(|v| v.checked_div(U192::from(SECONDS_IN_YEAR)))
            .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
            .ok_or(LendingError::MathOverflow)?;
        let total_interest = total_interest_precise.as_u64();
        let protocol_fee = U192::from(total_interest)
            .checked_mul(U192::from(self.protocol_fee_bps))
            .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
            .map(|v| v.as_u64())
            .ok_or(LendingError::MathOverflow)?;
        let lender_interest = total_interest.saturating_sub(protocol_fee);

        self.accrued_protocol_fees = self.accrued_protocol_fees.saturating_add(protocol_fee);
        self.total_deposits = self.total_deposits.saturating_add(lender_interest);
        self.total_loans = self.total_loans.saturating_add(total_interest);
        self.last_interest_update_timestamp = now;
        Ok(())
    }

    /// Calculates the current APY based on the kinked interest rate model.
    fn calculate_current_apy(&self, utilization_bps: u128) -> Result<u128> {
        let optimal_util_bps = self.optimal_utilization_bps as u128;
        if utilization_bps <= optimal_util_bps {
            let slope_factor = U192::from(utilization_bps)
                .checked_mul(U192::from(self.base_slope_bps))
                .and_then(|v| v.checked_div(U192::from(optimal_util_bps)))
                .map(|v| v.as_u128())
                .ok_or(LendingError::MathOverflow)?;
            self.base_borrow_rate_bps
                .checked_add(slope_factor)
                .ok_or_else(|| error!(LendingError::MathOverflow))
        } else {
            let rate_at_optimal = self
                .base_borrow_rate_bps
                .checked_add(self.base_slope_bps)
                .ok_or(LendingError::MathOverflow)?;
            let surplus_utilization = utilization_bps.saturating_sub(optimal_util_bps);
            let surplus_range = BASIS_POINTS_DIVISOR.saturating_sub(optimal_util_bps);
            let kink_slope_factor = U192::from(surplus_utilization)
                .checked_mul(U192::from(self.kink_slope_bps))
                .and_then(|v| v.checked_div(U192::from(surplus_range)))
                .map(|v| v.as_u128())
                .ok_or(LendingError::MathOverflow)?;
            rate_at_optimal
                .checked_add(kink_slope_factor)
                .ok_or_else(|| error!(LendingError::MathOverflow))
        }
    }
}

// --- 10. Helper Modules ---

pub mod cpi_utils {
    use super::*;
    pub fn transfer_from_user_checked<'info>(
        user: &AccountInfo<'info>,
        user_ata: &AccountInfo<'info>,
        destination: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: user_ata.clone(),
            to: destination.clone(),
            authority: user.clone(),
            mint: mint.clone(),
        };
        let cpi_context = CpiContext::new(token_program.clone(), cpi_accounts);
        token_interface::transfer_checked(cpi_context, amount, decimals)
    }
    pub fn transfer_from_vault_checked<'info>(
        from: &AccountInfo<'info>,
        to: &AccountInfo<'info>,
        mint: &AccountInfo<'info>,
        authority: &AccountInfo<'info>,
        token_program: &AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]],
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: from.clone(),
            to: to.clone(),
            authority: authority.clone(),
            mint: mint.clone(),
        };
        let cpi_context =
            CpiContext::new(token_program.clone(), cpi_accounts).with_signer(signer_seeds);
        token_interface::transfer_checked(cpi_context, amount, decimals)
    }
}

pub mod oracle {
    use super::*;
    #[repr(C, packed)]
    #[derive(Clone, Copy, Debug, Pod, Zeroable)]
    pub struct ChainlinkPriceFeed {
        pub description: [u8; 32],
        pub decimals: u8,
        _padding0: [u8; 7],
        pub round: u64,
        pub answer: i128,
        pub timestamp: i64,
    }

    pub fn get_price<'info>(
        pyth_feed_account: &AccountInfo<'info>,
        chainlink_feed_account: Option<&AccountInfo<'info>>, // <-- 核心修复：改为 Option
    ) -> Result<Price> {
        if let Ok(price) = get_price_from_pyth(pyth_feed_account) {
            return Ok(price);
        }

        if let Some(feed) = chainlink_feed_account {
            if let Ok(price) = get_price_from_chainlink(feed) {
                return Ok(price);
            }
        }

        Err(error!(LendingError::AllOraclesFailed))
    }

    fn get_price_from_pyth<'info>(price_feed_info: &AccountInfo<'info>) -> Result<Price> {
        // We assume the `test-oracle` feature is enabled for `anchor test`.
        // The production logic branch is kept for completeness.

        #[cfg(feature = "test-oracle")]
        {
            msg!("--- Debugging get_price_from_pyth (test-oracle feature enabled) ---");

            // --- 1. Attempt to borrow account data ---
            let account_data = match price_feed_info.try_borrow_data() {
                Ok(data) => data,
                Err(_) => {
                    msg!("Error: Failed to borrow account data.");
                    return Err(LendingError::InvalidPythAccount.into());
                }
            };
            msg!(
                "Successfully borrowed account data. Length: {}",
                account_data.len()
            );

            // --- 2. Attempt to cast bytes to SolanaPriceAccount using bytemuck ---
            let price_account: &SolanaPriceAccount = match bytemuck::try_from_bytes(&account_data) {
                Ok(acc) => acc,
                Err(e) => {
                    // Log the specific bytemuck error for detailed debugging
                    msg!(
                        "Error: Bytemuck failed to cast bytes to SolanaPriceAccount. Reason: {:?}",
                        e
                    );
                    return Err(LendingError::InvalidPythAccount.into());
                }
            };
            msg!("Successfully cast bytes to SolanaPriceAccount struct.");
            msg!(
                "Account data dump (first 64 bytes): {:?}",
                &account_data[..64]
            );
            msg!("Casted PriceAccount.magic: {}", price_account.magic);
            msg!("Casted PriceAccount.ver: {}", price_account.ver);
            msg!("Casted PriceAccount.expo: {}", price_account.expo);
            msg!("Casted PriceAccount.price: {}", price_account.agg.price);
            msg!("Casted PriceAccount.conf: {}", price_account.agg.conf);
            msg!("Casted PriceAccount.timestamp: {}", price_account.timestamp);

            // --- 3. Convert the account struct to a usable PriceFeed object ---
            let price_feed = price_account.to_price_feed(price_feed_info.key);
            msg!("Successfully converted SolanaPriceAccount to PriceFeed.");

            // --- 4. Check if the price is stale ---
            let current_timestamp = match Clock::get() {
                Ok(clock) => clock.unix_timestamp,
                Err(_) => {
                    msg!("Error: Failed to get clock.");
                    // This is a system-level error, but we can map it for clarity
                    return Err(LendingError::AllOraclesFailed.into());
                }
            };
            msg!("Current on-chain clock timestamp: {}", current_timestamp);
            msg!(
                "Price feed timestamp from account: {}",
                price_account.timestamp
            );

            let price = match price_feed
                .get_price_unchecked(current_timestamp, STALE_PRICE_THRESHOLD_SECONDS)
            {
                Some(p) => p,
                None => {
                    let time_diff = current_timestamp.saturating_sub(price_account.timestamp);
                    msg!("Error: get_price_unchecked returned None (price is too old).");
                    msg!(
                        "Time difference: {} seconds. Threshold: {} seconds.",
                        time_diff,
                        STALE_PRICE_THRESHOLD_SECONDS
                    );
                    return Err(LendingError::PythPriceTooOld.into());
                }
            };
            msg!("Validation Passed: Price is not stale.");
            msg!(
                "Retrieved Price object -> price: {}, conf: {}, expo: {}",
                price.price,
                price.conf,
                price.expo
            );

            // --- 5. Check if the price is positive ---
            if price.price <= 0 {
                msg!("Error: Price is not positive. Value: {}", price.price);
                return Err(LendingError::InvalidPythPrice.into());
            }
            msg!("Validation Passed: Price is positive.");

            // --- 6. Check the confidence interval ---
            let price_magnitude = price.price.unsigned_abs();

            // Use U192 for calculation to avoid overflow, as in the original code
            let max_conf = match U192::from(price_magnitude)
                .checked_mul(U192::from(MAX_CONFIDENCE_INTERVAL_BPS))
                .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
                .map(|v| v.as_u64())
            {
                Some(val) => val,
                None => {
                    msg!("Error: MathOverflow while calculating max_conf.");
                    return Err(LendingError::MathOverflow.into());
                }
            };

            msg!(
                "Max confidence allowed: {}. Actual confidence: {}",
                max_conf,
                price.conf
            );
            if price.conf > max_conf {
                msg!("Error: Confidence interval is too wide.");
                return Err(LendingError::PythConfidenceTooWide.into());
            }
            msg!("Validation Passed: Confidence is within limits.");

            msg!("--- Price successfully validated from Pyth ---");
            Ok(price)
        }

        #[cfg(not(feature = "test-oracle"))]
        {
            // This is the production code path
            msg!("--- Using get_price_from_pyth (production feature) ---");
            let price_feed = match SolanaPriceAccount::account_info_to_feed(price_feed_info) {
                Ok(feed) => feed,
                Err(e) => {
                    msg!("Error: account_info_to_feed failed: {:?}", e);
                    return Err(LendingError::InvalidPythAccount.into());
                }
            };

            let current_timestamp = Clock::get()?.unix_timestamp;

            let price = match price_feed
                .get_price_no_older_than(current_timestamp, STALE_PRICE_THRESHOLD_SECONDS)
            {
                Some(p) => p,
                None => return Err(LendingError::PythPriceTooOld.into()),
            };

            if price.price <= 0 {
                return Err(LendingError::InvalidPythPrice.into());
            }

            let price_magnitude = price.price.unsigned_abs();
            let max_conf = U192::from(price_magnitude)
                .checked_mul(U192::from(MAX_CONFIDENCE_INTERVAL_BPS))
                .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
                .map(|v| v.as_u64())
                .ok_or(LendingError::MathOverflow)?;

            if price.conf > max_conf {
                return Err(LendingError::PythConfidenceTooWide.into());
            }

            Ok(price)
        }
    }
    fn get_price_from_chainlink<'info>(chainlink_feed_info: &AccountInfo<'info>) -> Result<Price> {
        let data = chainlink_feed_info.try_borrow_data()?;
        require_gte!(
            data.len(),
            std::mem::size_of::<ChainlinkPriceFeed>(),
            LendingError::InvalidChainlinkPrice
        );
        let feed: &ChainlinkPriceFeed =
            bytemuck::from_bytes(&data[..std::mem::size_of::<ChainlinkPriceFeed>()]);
        require!(
            Clock::get()?.unix_timestamp.saturating_sub(feed.timestamp)
                < STALE_PRICE_THRESHOLD_SECONDS as i64,
            LendingError::ChainlinkPriceTooOld
        );
        require!(feed.answer > 0, LendingError::InvalidChainlinkPrice);
        Ok(Price {
            price: feed.answer as i64,
            conf: 0,
            expo: -(feed.decimals as i32),
            publish_time: feed.timestamp,
        })
    }
}

pub mod utils {
    use super::*;
    pub fn is_healthy(
        collateral: u64,
        debt: u64,
        price: Price,
        liquidation_threshold_bps: u64,
    ) -> Result<bool> {
        if debt == 0 {
            return Ok(true);
        }
        let collateral_value = calculate_asset_value(collateral, price)?;
        let debt_value = calculate_asset_value(debt, price)?;
        let max_debt_value = U192::from(collateral_value)
            .checked_mul(U192::from(liquidation_threshold_bps))
            .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
            .map(|v| v.as_u128())
            .ok_or(LendingError::MathOverflow)?;
        Ok(debt_value <= max_debt_value)
    }
    pub fn is_eligible_for_borrow(
        collateral: u64,
        new_debt: u64,
        price: Price,
        loan_to_value_bps: u64,
    ) -> Result<bool> {
        let collateral_value = calculate_asset_value(collateral, price)?;
        let new_debt_value = calculate_asset_value(new_debt, price)?;
        let max_borrow_value = U192::from(collateral_value)
            .checked_mul(U192::from(loan_to_value_bps))
            .and_then(|v| v.checked_div(U192::from(BASIS_POINTS_DIVISOR)))
            .map(|v| v.as_u128())
            .ok_or(LendingError::MathOverflow)?;
        Ok(new_debt_value <= max_borrow_value)
    }
    pub fn calculate_asset_value(amount: u64, price: Price) -> Result<u128> {
        let value_precise = U192::from(amount)
            .checked_mul(U192::from(price.price as u128))
            .ok_or(LendingError::MathOverflow)?;
        let final_value = if price.expo >= 0 {
            value_precise
                .checked_mul(U192::from(10u128.pow(price.expo as u32)))
                .ok_or(LendingError::MathOverflow)?
        } else {
            value_precise
                .checked_div(U192::from(10u128.pow(price.expo.abs() as u32)))
                .ok_or(LendingError::MathOverflow)?
        };
        final_value
            .checked_div(U192::from(PRECISION_DIVISOR))
            .map(|v| v.as_u128())
            .ok_or(error!(LendingError::MathOverflow))
    }
    pub fn calculate_amount_from_value(value: u128, price: Price) -> Result<u64> {
        let value_scaled = U192::from(value)
            .checked_mul(U192::from(PRECISION_DIVISOR))
            .ok_or(LendingError::MathOverflow)?;
        let amount_precise = if price.expo >= 0 {
            value_scaled
                .checked_div(U192::from(10u128.pow(price.expo as u32)))
                .ok_or(LendingError::MathOverflow)?
        } else {
            value_scaled
                .checked_mul(U192::from(10u128.pow(price.expo.abs() as u32)))
                .ok_or(LendingError::MathOverflow)?
        };
        amount_precise
            .checked_div(U192::from(price.price as u128))
            .map(|v| v.as_u64())
            .ok_or(error!(LendingError::MathOverflow))
    }
}

// --- 11. Error Codes ---
#[error_code]
pub enum LendingError {
    #[msg("Mathematical overflow during calculation.")]
    MathOverflow,
    #[msg("Invalid Pyth account provided.")]
    InvalidPythAccount,
    #[msg("The Pyth price feed is too old.")]
    PythPriceTooOld,
    #[msg("The Pyth price confidence interval is too wide.")]
    PythConfidenceTooWide,
    #[msg("The Chainlink price feed is too old.")]
    ChainlinkPriceTooOld,
    #[msg("Invalid Chainlink price (e.g., negative or zero).")]
    InvalidChainlinkPrice,
    #[msg("All available oracles failed to provide a valid price.")]
    AllOraclesFailed,
    #[msg("The Pyth price is invalid (e.g., negative or zero).")]
    InvalidPythPrice,
    #[msg("Collateral value is insufficient for this operation.")]
    InsufficientCollateral,
    #[msg("Not enough collateral deposited to withdraw this amount.")]
    InsufficientCollateralAmount,
    #[msg("The position is healthy and cannot be liquidated.")]
    PositionHealthy,
    #[msg("This operation would leave the position unhealthy.")]
    PositionWouldBecomeUnhealthy,
    #[msg("The signer is not the owner of the user position account.")]
    InvalidOwner,
    #[msg("Not enough collateral in the position for the liquidation seizure.")]
    InsufficientCollateralForLiquidation,
    #[msg("The transaction amount cannot be zero.")]
    ZeroAmount,
    #[msg("The maximum number of asset pools has been reached.")]
    MaxAssetsExceeded,
    #[msg("The asset pool has insufficient liquidity for this operation.")]
    InsufficientLiquidity,
    #[msg("The signer does not match the approved delegatee for this credit line.")]
    DelegationMismatch,
    #[msg("The requested borrow amount exceeds the delegated credit line.")]
    DelegationExceeded,
    #[msg("The operation provided in the batch transaction is invalid.")]
    InvalidOperation,
    #[msg("The delegation is currently in use and cannot be revoked.")]
    DelegationIsActive,
    #[msg("Flash loans are not enabled for this asset pool.")]
    FlashLoanNotAvailable,
    #[msg("Flash loan callback cannot be the lending program itself.")]
    FlashLoanReentrancy,
    #[msg("The flash loan was not fully repaid with the required fee.")]
    FlashLoanRepaymentFailed,
    #[msg("The provided collateral mint account does not match the one in the collateral pool.")]
    CollateralMintMismatch,
    #[msg("The provided loan mint account does not match the one in the loan pool.")]
    LoanMintMismatch,
    #[msg("The protocol is currently paused by governance.")]
    ProtocolPaused,
    #[msg("The protocol is not active. No new positions or loans can be created.")]
    ProtocolNotActive,
    #[msg("A liquidator cannot liquidate their own position.")]
    CannotLiquidateSelf,
    #[msg("The provided oracle account is not valid or recognized.")]
    InvalidOracleAccount,
    #[msg("Loan-to-value cannot be greater than the liquidation threshold.")]
    InvalidLtv,
    #[msg("Liquidation threshold must be less than 100%.")]
    InvalidLiquidationThreshold,
    #[msg("Optimal utilization must be less than 100%.")]
    InvalidOptimalUtilization,
    #[msg("The provided asset vault account is invalid for this pool.")]
    InvalidAssetVault,
    #[msg("The provided asset Pool account is invalid for this pool.")]
    InvalidAssetPool,
    #[msg("The provided asset Mint account is invalid for this pool.")]
    InvalidAssetMint,
}
