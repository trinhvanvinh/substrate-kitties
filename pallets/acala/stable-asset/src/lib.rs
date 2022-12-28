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
	use crate::{PoolTokenIndex, RedeemProportionResult};
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

		
	}
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

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
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
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
