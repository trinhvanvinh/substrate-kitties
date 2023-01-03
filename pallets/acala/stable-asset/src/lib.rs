#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

// #[cfg(test)]
// mod mock;

// #[cfg(test)]
// mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use crate::traits::StableAsset;
use frame_support::codec::{Decode, Encode};
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::ensure;
use frame_support::traits::fungibles::{Inspect, Mutate, Transfer};
use frame_support::traits::Get;
use scale_info::TypeInfo;
use sp_core::U512;
use sp_runtime::{
	traits::{AccountIdConversion, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero},
	SaturatedConversion,
};
use sp_std::result::Result;

pub type PoolTokenIndex = u32;
pub type StableAssetPoolId = u32;
const NUMBER_OF_ITERATIONS_TO_CONVERGE: i32 = 255;

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, TypeInfo)]
pub struct StableAssetPoolInfo<AssetId, AtLeast64BitUnsigned, Balance, AccountId, BlockNumber> {
	pub pool_asset: AssetId,
	pub assets: Vec<AssetId>,
	pub precisions: Vec<AtLeast64BitUnsigned>,
	pub mint_fee: AtLeast64BitUnsigned,
	pub swap_fee: AtLeast64BitUnsigned,
	pub redeem_fee: AtLeast64BitUnsigned,
	pub total_supply: Balance,
	pub a: AtLeast64BitUnsigned,
	pub a_block: BlockNumber,
	pub future_a: AtLeast64BitUnsigned,
	pub future_a_block: BlockNumber,
	pub balances: Vec<Balance>,
	pub fee_recipient: AccountId,
	pub account_id: AccountId,
	pub yield_recipient: AccountId,
	pub precision: AtLeast64BitUnsigned,
}

pub mod traits {
	use crate::{
		PoolTokenIndex, RedeemProportionResult, StableAssetPoolId, StableAssetPoolInfo, SwapResult,
	};
	use frame_support::dispatch::{DispatchError, DispatchResult};
	use sp_std::prelude::*;

	pub trait ValidateAssetId<AssetId> {
		fn validate(a: AssetId) -> bool;
	}

	pub trait StableAsset {
		type AssetId;
		type AtLeast64BitUnsigned;
		type Balance;
		type AccountId;
		type BlockNumber;

		fn pool_count() -> StableAssetPoolId;

		fn pool(
			id: StableAssetPoolId,
		) -> Option<
			StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		>;

		fn create_pool(
			pool_asset: Self::AssetId,
			assets: Vec<Self::AssetId>,
			precisions: Vec<Self::AtLeast64BitUnsigned>,
			mint_fee: Self::AtLeast64BitUnsigned,
			swap_fee: Self::AtLeast64BitUnsigned,
			redeem_fee: Self::AtLeast64BitUnsigned,
			initial_a: Self::AtLeast64BitUnsigned,
			fee_recipient: Self::AccountId,
			yield_recipient: Self::AccountId,
			precision: Self::AtLeast64BitUnsigned,
		) -> DispatchResult;

		fn mint(
			who: Self::AccountId,
			pool_id: StableAssetPoolId,
			amounts: Vec<Self::Balance>,
			min_mint_amount: Self::Balance,
		) -> DispatchResult;

		fn swap(
			who: Self::AccountId,
			pool_id: StableAssetPoolId,
			i: PoolTokenIndex,
			j: PoolTokenIndex,
			dx: Self::Balance,
			min_dy: Self::Balance,
			asset_length: u32,
		) -> Result<(Self::Balance, Self::Balance), DispatchError>;

		fn redeem_proportion(
			who: Self::AccountId,
			pool_id: StableAssetPoolId,
			amount: Self::Balance,
			min_redeem_amounts: Vec<Self::Balance>,
		) -> DispatchResult;

		fn redeem_single(
			who: Self::AccountId,
			pool_id: StableAssetPoolId,
			amount: Self::Balance,
			i: PoolTokenIndex,
			min_redeem_amount: Self::Balance,
			asset_length: u32,
		) -> Result<(Self::Balance, Self::Balance), DispatchError>;

		fn redeem_multi(
			who: Self::AccountId,
			pool_id: StableAssetPoolId,
			amounts: Vec<Self::Balance>,
			max_redeem_amount: Self::Balance,
		) -> DispatchResult;

		fn collect_fee(
			pool_id: StableAssetPoolId,
			pool_info: StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		) -> DispatchResult;

		fn update_balance(
			pool_id: StableAssetPoolId,
			pool_info: StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		) -> DispatchResult;

		fn collect_yield(
			pool_id: StableAssetPoolId,
			pool_info: StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		) -> DispatchResult;

		fn modify_a(
			pool_id: StableAssetPoolId,
			a: Self::AtLeast64BitUnsigned,
			future_a_block: Self::BlockNumber,
		) -> DispatchResult;

		fn get_collect_yeild_amount(
			pool_info: StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		) -> Option<
			StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		>;

		fn get_balance_update_amount(
			pool_info: StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		) -> Option<
			StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
		>;

		fn get_redeem_proportion_amount(
			pool_info: StableAssetPoolInfo<
				Self::AssetId,
				Self::AtLeast64BitUnsigned,
				Self::Balance,
				Self::AccountId,
				Self::BlockNumber,
			>,
			amount_bal: Self::Balance,
		) -> Option<RedeemProportionResult<Self::Balance>>;

		fn get_best_route(
			input_asset: Self::AssetId,
			output_asset: Self::AssetId,
			input_amount: Self::Balance,
		) -> Option<(StableAssetPoolId, PoolTokenIndex, PoolTokenIndex, Self::Balance)>;

		fn get_swap_output_amount(
			pool_id: StableAssetPoolId,
			input_index: PoolTokenIndex,
			output_index: PoolTokenIndex,
			dx_bal: Self::Balance,
		) -> Option<SwapResult<Self::Balance>>;

		fn get_swap_input_amount(
			pool_id: StableAssetPoolId,
			input_index: PoolTokenIndex,
			output_index: PoolTokenIndex,
			dy_bal: Self::Balance,
		) -> Option<SwapResult<Self::Balance>>;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::{PoolTokenIndex, StableAssetPoolId, StableAssetPoolInfo};
	use crate::traits::{StableAsset, ValidateAssetId};
	use frame_support::pallet_prelude::*;
	use frame_support::traits::tokens::fungibles;
	use frame_support::{
		dispatch::{Codec, DispatchResult},
		traits::EnsureOrigin,
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{
		traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, One, Zero},
		FixedPointOperand,
	};
	use sp_std::prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type AssetId: Parameter + Ord + Copy;
		type Balance: Parameter + Codec + Copy + Ord + From<Self::AtLeast64BitUnsigned> + Zero;
		type Assets: fungibles::Inspect<
			Self::AccountId,
			AssetId = Self::AssetId,
			Balance = Self::Balance,
		>;
		type AtLeast64BitUnsigned: Parameter
			+ CheckedAdd
			+ CheckedSub
			+ CheckedMul
			+ CheckedDiv
			+ Copy
			+ Eq
			+ Ord
			+ From<Self::Balance>
			+ From<u8>
			+ From<u128>
			+ From<Self::BlockNumber>
			+ TryFrom<usize>
			+ Zero
			+ One
			+ FixedPointOperand;
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		#[pallet::constant]
		type FeePrecision: Get<Self::AtLeast64BitUnsigned>;
		#[pallet::constant]
		type SwapExactOverAmount: Get<Self::AtLeast64BitUnsigned>;
		#[pallet::constant]
		type Aprecision: Get<Self::AtLeast64BitUnsigned>;
		#[pallet::constant]
		type PoolAssetLimit: Get<u32>;
		type EnsurePoolAssetId: ValidateAssetId<Self::AssetId>;

		type ListingOrigin: EnsureOrigin<Self::RuntimeOrigin>;
	}
	#[pallet::storage]
	#[pallet::getter(fn pool_count)]
	pub type PoolCount<T: Config> = StorageValue<_, StableAssetPoolId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn pools)]
	pub type Pools<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		StableAssetPoolId,
		StableAssetPoolInfo<
			T::AssetId,
			T::AtLeast64BitUnsigned,
			T::Balance,
			T::AccountId,
			T::BlockNumber,
		>,
	>;

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),

		CreatePool {
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			swap_id: T::AccountId,
			pallet_id: T::AccountId,
		},
		Minted {
			minter: T::AccountId,
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			input_amounts: Vec<T::Balance>,
			min_output_amount: T::Balance,
			balances: Vec<T::Balance>,
			total_supply: T::Balance,
			fee_amount: T::Balance,
			output_amount: T::Balance,
		},
		TokenSwapped {
			swapper: T::AccountId,
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			input_asset: T::AssetId,
			output_asset: T::AssetId,
			input_amount: T::Balance,
			min_output_amount: T::Balance,
			balances: Vec<T::Balance>,
			total_supply: T::Balance,
			output_amount: T::Balance,
		},
		RedeemedProportion {
			redeemer: T::AccountId,
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			input_amount: T::Balance,
			min_output_amounts: Vec<T::Balance>,
			balances: Vec<T::Balance>,
			total_supply: T::Balance,
			fee_amount: T::Balance,
			output_amounts: Vec<T::Balance>,
		},

		RedeemedSingle {
			redeemer: T::AccountId,
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			input_amount: T::Balance,
			output_asset: T::AssetId,
			mint_output_amount: T::Balance,
			balances: Vec<T::Balance>,
			total_supply: T::Balance,
			fee_amount: T::Balance,
			output_amount: T::Balance,
		},
		RedeemedMulti {
			redeemer: T::AccountId,
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			output_amounts: Vec<T::Balance>,
			max_input_asset: T::Balance,
			balances: Vec<T::Balance>,
			total_supply: T::Balance,
			fee_amount: T::Balance,
			input_amount: T::Balance,
		},
		BalanceUpdated {
			pool_id: StableAssetPoolId,
			old_balances: Vec<T::Balance>,
			new_balances: Vec<T::Balance>,
		},
		YieldCollected {
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			old_total_supply: T::Balance,
			new_total_suplly: T::Balance,
			who: T::AccountId,
			amount: T::Balance,
		},

		FeeCollected {
			pool_id: StableAssetPoolId,
			a: T::AtLeast64BitUnsigned,
			old_balances: Vec<T::Balance>,
			new_balances: Vec<T::Balance>,
			old_total_supply: T::Balance,
			new_total_supply: T::Balance,
			who: T::AccountId,
			amount: T::Balance,
		},
		AModified {
			pool_id: StableAssetPoolId,
			value: T::AtLeast64BitUnsigned,
			time: T::BlockNumber,
		},
		FeeModified {
			pool_id: StableAssetPoolId,
			mint_fee: T::AtLeast64BitUnsigned,
			swap_fee: T::AtLeast64BitUnsigned,
			redeem_fee: T::AtLeast64BitUnsigned,
		},
		RecipientModified {
			pool_id: StableAssetPoolId,
			fee_recipient: T::AccountId,
			yield_recipient: T::AccountId,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		InconsistentStorage,
		InvalidPoolAsset,
		ArgumentsMismatch,
		ArgumentsError,
		PoolNotFound,
		Math,
		InvalidPoolValue,
		MintUnderMin,
		SwapUnderMin,
		RedeemUnderMin,
		RedeemOverMax,
	}

	pub struct MintResult<T: Config> {
		pub mint_amount: T::Balance,
		pub fee_amount: T::Balance,
		pub balances: Vec<T::Balance>,
		pub total_supply: T::Balance,
	}

	pub struct SwapResult<Balance> {
		pub dx: Balance,
		pub dy: Balance,
		pub y: Balance,
		pub balance_i: Balance,
	}

	pub struct RedeemProportionResult<Balance> {
		pub amounts: Vec<Balance>,
		pub balances: Vec<Balance>,
		pub fee_amount: Balance,
		pub total_supply: Balance,
		pub redeem_amount: Balance,
	}

	pub struct RedeemSingleResult<T: Config> {
		pub dy: T::Balance,
		pub fee_amount: T::Balance,
		pub total_supply: T::Balance,
		pub balances: Vec<T::Balance>,
		pub redeem_amount: T::Balance,
	}

	pub struct RedeemMultiResult<T: Config> {
		pub redeem_amount: T::Balance,
		pub fee_amount: T::Balance,
		pub balances: Vec<T::Balance>,
		pub total_supply: T::Balance,
		pub burn_amount: T::Balance,
	}

	pub struct PendingFeeResult<T: Config> {
		pub fee_amount: T::Balance,
		pub balances: Vec<T::Balance>,
		pub total_supply: T::Balance,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/main-docs/build/origins/
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	pub fn convert_vec_number_to_balance(numbers: Vec<T::AtLeast64BitUnsigned>) -> Vec<T::Balance> {
		numbers.into_iter().map(|x| x.into()).collect()
	}

	pub fn convert_vec_balance_to_number(
		balances: Vec<T::Balance>,
	) -> Vec<T::AtLeast64BitUnsigned> {
		balances.into_iter().map(|x| x.into()).collect()
	}

	pub fn get_a(
		a0: T::AtLeast64BitUnsigned,
		t0: T::BlockNumber,
		a1: T::AtLeast64BitUnsigned,
		t1: T::BlockNumber,
	) -> Option<T::AtLeast64BitUnsigned> {
		let current_block = frame_system::Pallet::<T>::block_number();

		if current_block < t1 {
			let time_diff: T::AtLeast64BitUnsigned = current_block.checked_sub(&t0)?.into();
			let time_diff_div: T::AtLeast64BitUnsigned = t1.checked_sub(&t0)?.into();
			if a1 > a0 {
				let diff = a1.checked_sub(&a0)?;
				let amount = diff.checked_mul(&time_diff)?.checked_div(&time_diff_div)?;
				Some(a0.checked_add(&amount)?)
			} else {
				let diff = a0.checked_sub(&a1)?;
				let amount = diff.checked_mul(&time_diff)?.checked_div(&time_diff_div)?;
				Some(a0.checked_sub(&amount)?)
			}
		} else {
			Some(a1)
		}
	}

	pub fn get_d(
		balances: &[T::AtLeast64BitUnsigned],
		a: T::AtLeast64BitUnsigned,
	) -> Option<T::AtLeast64BitUnsigned> {
		let zero: U512 = U512::from(0u128);
		let one: U512 = U512::from(1u128);
		let mut sum: U512 = U512::from(0u128);
		let mut ann: U512 = U512::from(a.saturated_into::<u128>());
		let balance_size: U512 = U512::from(balances.len());
		let a_precision_u256: U512 = U512::from(T::Aprecision::get().saturated_into::<u128>());
		for x in balances.iter() {
			let balance: u128 = (*x).saturated_into::<u128>();
			sum = sum.checked_add(balance.into())?;
			ann = ann.checked_mul(balance_size)?;
		}
		if sum == zero {
			return Some(Zero::zero());
		}

		let prev_d: U512;
		let d: U512 = sum;

		for _i in 0..NUMBER_OF_ITERATIONS_TO_CONVERGE {
			let mut p_d: U512 = d;
			for x in balances.iter() {
				let balance: u128 = (*x).saturated_into::<u128>();
				let div_op = U512::from(balance).checked_mul(balance_size)?;
				p_d = p_d.checked_mul(d)?.checked_div(div_op)?;
			}
			prev_d = d;
			let t1: U512 = p_d.checked_mul(balance_size)?;
			let t2: U512 = balance_size.checked_add(one)?.checked_mul(p_d)?;
			let t3: U512 = ann
				.checked_sub(a_precision_u256)?
				.checked_mul(d)?
				.checked_div(a_precision_u256)?
				.checked_add(t2)?;
			d = ann
				.checked_mul(sum)?
				.checked_div(a_precision_u256)?
				.checked_add(t1)?
				.checked_mul(d)?
				.checked_div(t3)?;

			if d > prev_d {
				if d - prev_d <= one {
					break;
				}
			} else if prev_d - d <= one {
				break;
			}
		}
		let result: u128 = u128::try_from(d).ok()?;
		Some(result.into())
	}

	pub fn get_y(
		balances: &[T::AtLeast64BitUnsigned],
		token_index: PoolTokenIndex,
		target_d: T::AtLeast64BitUnsigned,
		amplitude: T::AtLeast64BitUnsigned,
	) -> Option<T::AtLeast64BitUnsigned> {
		let one: U512 = U512::from(1u128);
		let two: U512 = U512::from(2u128);
		let mut c: U512 = U512::from(target_d.saturated_into::<u128>());
		let mut sum: U512 = U512::from(0u128);
		let mut ann: U512 = U512::from(amplitude.saturated_into::<u128>());
		let balance_size: U512 = U512::from(balances.len());
		let target_d_u256: U512 = U512::from(target_d.saturated_into::<u128>());
		let a_precision_u256: U512 = U512::from(T::Aprecision::get().saturated_into::<u128>());
		for (i, balance_ref) in balances.iter().enumerate() {
			let balance: U512 = U512::from((*balance_ref).saturated_into::<u128>());
			ann = ann.checked_mul(balance_size)?;
			let token_index_usize = token_index as usize;
			if i == token_index_usize {
				continue;
			}
			sum = sum.checked_add(balance)?;
			let div_op: U512 = balance.checked_mul(balance_size)?;
			c = c.checked_mul(target_d_u256)?.checked_div(div_op)?
		}

		c = c
			.checked_mul(target_d_u256)?
			.checked_mul(a_precision_u256)?
			.checked_div(ann.checked_mul(balance_size)?)?;
		let b: U512 =
			sum.checked_add(target_d_u256.checked_mul(a_precision_u256)?.checked_div(ann)?)?;
		let mut prev_y: U512;
		let mut y: U512 = target_d_u256;

		for _i in 0..NUMBER_OF_ITERATIONS_TO_CONVERGE {
			prev_y = y;
			y = y
				.checked_mul(y)?
				.checked_add(c)?
				.checked_div(y.checked_mul(two)?.checked_add(b)?.checked_sub(target_d_u256)?)?;
			if y > prev_y {
				if y - prev_y <= one {
					break;
				}
			} else if prev_y - y <= one {
				break;
			}
		}
		let result: u128 = u128::try_from(y).ok()?;
		Some(result.into())
	}

	pub fn get_mint_amount(
		pool_info: &StableAssetPoolInfo<
			T::AssetId,
			T::AtLeast64BitUnsigned,
			T::Balance,
			T::AccountId,
			T::BlockNumber,
		>,
		amounts_bal: &[T::Balance],
	) -> Result<MintResult<T>, Error<T>> {
		if pool_info.balances.len() != amounts_bal.len() {
			return Err(Error::<T>::ArgumentsMismatch);
		}
		let amounts = Self::convert_vec_balance_to_number(amounts_bal.to_vec());
		let a: T::AtLeast64BitUnsigned = Self::get_a(
			pool_info.a,
			pool_info.a_block,
			pool_info.future_a,
			pool_info.future_a_block,
		)
		.ok_or(Error::<T>::Math)?;
		let old_d: T::AtLeast64BitUnsigned = pool_info.total_supply.into();
		let zero: T::AtLeast64BitUnsigned = Zero::zero();
		let fee_denominator: T::AtLeast64BitUnsigned = T::FeePrecision::get();

		let mut balances: Vec<T::AtLeast64BitUnsigned> =
			Self::convert_vec_balance_to_number(pool_info.balances.clone());
		for i in 0..balances.len() {
			if amounts[i] == zero {
				if old_d == zero {
					return Err(Error::<T>::ArgumentsError);
				}
				continue;
			}
			let result: T::AtLeast64BitUnsigned = balances[i]
				.checked_add(
					&amounts[i].checked_mul(&pool_info.precisions[i]).ok_or(Error::<T>::Math)?,
				)
				.ok_or(Error::<T>::Math)?;
			balances[i] = result;
		}
		let new_d: T::AtLeast64BitUnsigned = Self::get_d(&balances, a).ok_or(Error::<T>::Math)?;
		let mut mint_amount: T::AtLeast64BitUnsigned =
			new_d.checked_sub(&old_d).ok_or(Error::<T>::Math)?;
		let mut fee_amount: T::AtLeast64BitUnsigned = zero;
		let mint_fee: T::AtLeast64BitUnsigned = pool_info.mint_fee;

		if pool_info.mint_fee > zero {
			fee_amount = mint_amount
				.checked_mul(&mint_fee)
				.ok_or(Error::<T>::Math)?
				.checked_div(&fee_denominator)
				.ok_or(Error::<T>::Math)?;
			mint_amount = mint_amount.checked_sub(&fee_amount).ok_or(Error::<T>::Math)?;
		}

		Ok(MintResult {
			mint_amount: mint_amount.into(),
			fee_amount: fee_amount.into(),
			balances: Self::convert_vec_number_to_balance(balances),
			total_supply: new_d.into(),
		})
	}

	pub fn get_swap_amount() -> Result<SwapResult<T::Balance>, Error<T>> {

		Ok(())
	}

	pub fn get_balance_update_amount(
		pool_info: StableAssetPoolInfo<
			T::AssetId,
			T::AtLeast64BitUnsigned,
			T::Balance,
			T::AccountId,
			T::BlockNumber,
		>,
	) -> Result<
		StableAssetPoolInfo<
			T::AssetId,
			T::AtLeast64BitUnsigned,
			T::Balance,
			T::AccountId,
			T::BlockNumber,
		>,
		Error<T>,
	> {
		let mut updated_balances = pool_info.balances;
		for (i, balance) in updated_balances.iter_mut().enumerate() {
			let balance_of: T::AtLeast64BitUnsigned =
				T::Assets::balance(pool_info.assets[i], &pool_info.account_id).into();
			*balance =
				balance_of.checked_mul(&pool_info.precisions[i]).ok_or(Error::<T>::Math)?.into();
		}
		let mut cloned_stable_asset_info = pool_info;
		cloned_stable_asset_info.balances = updated_balances;

		Ok(cloned_stable_asset_info)
	}

	pub fn get_collect_yield_amount(
		pool_info: StableAssetPoolInfo<
			T::AssetId,
			T::AtLeast64BitUnsigned,
			T::Balance,
			T::AccountId,
			T::BlockNumber,
		>,
	) -> Result<
		StableAssetPoolInfo<
			T::AssetId,
			T::AtLeast64BitUnsigned,
			T::Balance,
			T::AccountId,
			T::BlockNumber,
		>,
		Error<T>,
	> {
		//let
		Ok()
	}
}

impl<T: Config> StableAsset for Pallet<T> {
	type AssetId = T::AssetId;
	type AtLeast64BitUnsigned = T::AtLeast64BitUnsigned;
	type Balance = T::Balance;
	type AccountId = T::AccountId;
	type BlockNumber = T::BlockNumber;

	fn pool_count() -> StableAssetPoolId {
		PoolCount::<T>::get()
	}

	fn pool(
		id: StableAssetPoolId,
	) -> Option<
		StableAssetPoolInfo<
			Self::AssetId,
			Self::AtLeast64BitUnsigned,
			Self::Balance,
			Self::AccountId,
			Self::BlockNumber,
		>,
	> {
		Pools::<T>::get(id)
	}

	fn update_balance(
		pool_id: StableAssetPoolId,
		pool_info: StableAssetPoolInfo<
			Self::AssetId,
			Self::AtLeast64BitUnsigned,
			Self::Balance,
			Self::AccountId,
			Self::BlockNumber,
		>,
	) -> DispatchResult {
		let old_balances = pool_info.balances;
		let new_balance_pool_info = Self::get_balance_update_amount(pool_info)?;
		pool_info.balances = new_balance_pool_info.balances;
		Self::deposit_event(Event::BalanceUpdated {
			pool_id,
			old_balances,
			new_balances: pool_info.balances,
		});
		Ok(())
	}

	fn collect_yield(
		pool_id: StableAssetPoolId,
		pool_info: StableAssetPoolInfo<
			Self::AssetId,
			Self::AtLeast64BitUnsigned,
			Self::Balance,
			Self::AccountId,
			Self::BlockNumber,
		>,
	) -> DispatchResult {
		let old_total_supply = pool_info.total_supply;
		Self::update_balance(pool_id, pool_info);

		let updated_total_supply_pool_info = Self::get_collect_yield_amount();

		Ok(())
	}

	fn get_balance_update_amount(
		pool_info: StableAssetPoolInfo<
			Self::AssetId,
			Self::AtLeast64BitUnsigned,
			Self::Balance,
			Self::AccountId,
			Self::BlockNumber,
		>,
	) -> Option<
		StableAssetPoolInfo<
			Self::AssetId,
			Self::AtLeast64BitUnsigned,
			Self::Balance,
			Self::AccountId,
			Self::BlockNumber,
		>,
	> {
		Self::get_balance_update_amount(pool_info).ok()
	}
}
