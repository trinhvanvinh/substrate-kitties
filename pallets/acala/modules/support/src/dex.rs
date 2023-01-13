use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::traits::Get;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_core::H160;
use sp_runtime::{DispatchError, RuntimeDebug};
use sp_std::{cmp::PartialEq, prelude::*, result::Result};
use stable_asset::{PoolTokenIndex, StableAssetPoolId};

#[derive(Clone, Decode, Encode)]
pub enum SwapLimit<Balance> {
	ExactSupply(Balance, Balance),
	ExactTarget(Balance, Balance),
}
#[derive(
	Clone, Decode, Encode, TypeInfo, Eq, PartialEq, PartialOrd, Ord, RuntimeDebug, MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AggregatedSwapPath<CurrencyId> {
	Dex(Vec<CurrencyId>),
	Taiga(StableAssetPoolId, PoolTokenIndex, PoolTokenIndex),
}

pub trait DEXManager<AccountId, Balance, CurrencyId> {
	fn get_liquidity_pool(
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
	) -> (Balance, Balance);
	fn get_liquidity_token_address(
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
	) -> Option<H160>;

	fn get_swap_amount(
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Option<(Balance, Balance)>;

	fn get_best_price_swap_path(
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
		alternative_path_join_list: Vec<Vec<CurrencyId>>,
	) -> Option<(Vec<CurrencyId>, Balance, Balance)>;

	fn swap_with_specific_path(
		who: AccountId,
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError>;

	fn add_liquidity(
		who: AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		max_amount_a: Balance,
		max_amount_b: Balance,
		min_share_increment: Balance,
		stake_increment_share: bool,
	) -> Result<(Balance, Balance, Balance), DispatchError>;

	fn remove_liquidity(
		who: AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		remove_share: Balance,
		min_withdraw_a: Balance,
		min_withdraw_b: Balance,
		by_unstake: bool,
	) -> Result<(Balance, Balance), DispatchError>;
}

pub trait Swap<AccountId, Balance, CurrencyId>
where
	CurrencyId: Clone,
{
	fn get_swap_amount(
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Option<(Balance, Balance)>;

	fn swap(
		who: AccountId,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError>;

	fn swap_by_path(
		who: AccountId,
		swap_path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		let aggregated_swap_path = AggregatedSwapPath::Dex(swap_path.to_vec());
		Self::swap_by_aggregated_path(who, &[aggregated_swap_path], limit)
	}
	fn swap_by_aggregated_path(
		who: AccountId,
		swap_path: &[AggregatedSwapPath<CurrencyId>],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError>;
}

pub enum SwapError {
	CannotSwap,
}

impl Into<DispatchError> for SwapError {
	fn into(self) -> DispatchError {
		DispatchError::Other("Cannot Swap")
	}
}

// dex wrapper
pub struct SpecificJointsSwap<Dex, Joints>(sp_std::marker::PhantomData<(Dex, Joints)>);

impl<AccountId, Balance, CurrencyId, Dex, Joints> Swap<AccountId, Balance, CurrencyId>
	for SpecificJointsSwap<Dex, Joints>
where
	Dex: DEXManager<AccountId, Balance, CurrencyId>,
	Joints: Get<Vec<Vec<CurrencyId>>>,
	Balance: Clone,
	CurrencyId: Clone,
{
	fn get_swap_amount(
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Option<(Balance, Balance)> {
		<Dex as DEXManager<AccountId, Balance, CurrencyId>>::get_best_price_swap_path(
			supply_currency_id,
			target_currency_id,
			limit,
			Joints::get(),
		)
		.map(|(_, supply_amount, target_amount)| (supply_amount, target_amount))
	}

	fn swap(
		who: AccountId,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		let path = <Dex as DEXManager<AccountId, Balance, CurrencyId>>::get_best_price_swap_path(
			supply_currency_id,
			target_currency_id,
			limit.clone(),
			Joints::get(),
		)
		.ok_or_else(|| Into::<DispatchError>::into(SwapError::CannotSwap))?
		.0;

		<Dex as DEXManager<AccountId, Balance, CurrencyId>>::swap_with_specific_path(
			who, &path, limit,
		)
	}

	fn swap_by_path(
		who: AccountId,
		swap_path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		<Dex as DEXManager<AccountId, Balance, CurrencyId>>::swap_with_specific_path(
			who, swap_path, limit,
		)
	}

	fn swap_by_aggregated_path(
		who: AccountId,
		swap_path: &[AggregatedSwapPath<CurrencyId>],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		Err(Into::<DispatchError>::into(SwapError::CannotSwap))
	}
}

impl<AccountId, CurrencyId, Balance> DEXManager<AccountId, Balance, CurrencyId> for ()
where
	Balance: Default,
{
	fn get_liquidity_pool(
		currency_id_a: CurrencyId,
		currncy_id_b: CurrencyId,
	) -> (Balance, Balance) {
		Default::default()
	}

	fn get_liquidity_token_address(
		currency_id_a: CurrencyId,
		currncy_id_b: CurrencyId,
	) -> Option<H160> {
		Some(Default::default())
	}

	fn get_swap_amount(
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Option<(Balance, Balance)> {
		Some(Default::default())
	}

	fn get_best_price_swap_path(
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
		alternative_path_join_list: Vec<Vec<CurrencyId>>,
	) -> Option<(Vec<CurrencyId>, Balance, Balance)> {
		Some(Default::default())
	}

	fn swap_with_specific_path(
		who: AccountId,
		path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		Ok(Default::default())
	}

	fn add_liquidity(
		who: AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		max_amount_a: Balance,
		max_amount_b: Balance,
		min_share_increment: Balance,
		stake_increment_share: bool,
	) -> Result<(Balance, Balance, Balance), DispatchError> {
		Ok(Default::default())
	}

	fn remove_liquidity(
		who: AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		remove_share: Balance,
		min_withdraw_a: Balance,
		min_withdraw_b: Balance,
		by_unstake: bool,
	) -> Result<(Balance, Balance), DispatchError> {
		Ok(Default::default())
	}
}
