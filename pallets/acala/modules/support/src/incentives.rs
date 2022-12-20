use crate::Rate;
use primitives::CurrencyId;

use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{DispatchResult, RuntimeDebug};

#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub enum PoolId {
	Loans(CurrencyId),
	Dex(CurrencyId),
}

pub trait IncentivesManager<AccountId, Balance, CurrencyId, PoolId> {
	fn get_incentive_reward_amount(pool_id: PoolId, currency_id: CurrencyId) -> Balance;
	fn deposit_dex_share(
		who: AccountId,
		lp_currency_id: CurrencyId,
		amount: Balance,
	) -> DispatchResult;
	fn withdraw_dex_share(
		who: AccountId,
		lp_currency_id: CurrencyId,
		amount: Balance,
	) -> DispatchResult;
	fn claim_rewards(who: AccountId, pool_id: PoolId) -> DispatchResult;
	fn get_claim_reward_deduction_rate(pool_id: PoolId) -> Rate;
	fn get_pending_rewards(
		pool_id: PoolId,
		who: AccountId,
		reward_currency: Vec<CurrencyId>,
	) -> DispatchResult;
}

pub trait DEXIncentives<AccountId, CurrencyId, Balance> {
	fn do_deposit_dex_share(
		who: AccountId,
		lp_currency_id: CurrencyId,
		amount: Balance,
	) -> DispatchResult;
	fn do_withdraw_dex_share(
		who: AccountId,
		lp_currency_id: CurrencyId,
		amount: Balance,
	) -> DispatchResult;
}

#[cfg(feature = "std")]
impl<AccountId, CurrencyId, Balance> DEXIncentives<AccountId, CurrencyId, Balance> for () {
	fn do_deposit_dex_share(_: AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		Ok(())
	}

	fn do_withdraw_dex_share(_: AccountId, _: CurrencyId, _: Balance) -> DispatchResult {
		Ok(())
	}
}
