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
use frame_support::{pallet_prelude::*, PalletId};
use frame_system::pallet_prelude::*;
use module_support::{DEXIncentives, Erc20InfoMapping, ExchangeRate};
use module_traits::{Happened, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId, TradingPair};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[derive(Decode, Encode, MaxEncodedLen, TypeInfo, Clone, RuntimeDebug, Copy, PartialEq, Eq)]
pub struct ProvisioningParameters<Balance, BlockNumber> {
	min_contribution: (Balance, Balance),
	target_provision: (Balance, Balance),
	accumulated_provision: (Balance, Balance),
	not_before: BlockNumber,
}
#[derive(Decode, Encode, MaxEncodedLen, TypeInfo, Clone, RuntimeDebug, Copy, PartialEq, Eq)]
pub enum TradingPairStatus<Balance, BlockNumber> {
	Disabled,
	Provisioning(ProvisioningParameters<Balance, BlockNumber>),
	Enabled,
}

impl<Balance, BlockNumber> Default for TradingPairStatus<Balance, BlockNumber> {
	fn default() -> Self {
		Self::Disabled
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: MultiCurrencyExtended<
			Self::AccountId,
			CurrencyId = CurrencyId,
			Balance = Balance,
		>;
		#[pallet::constant]
		type GetExchangeFee: Get<(u32, u32)>;

		#[pallet::constant]
		type TradingPathLimit: Get<u32>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type Erc20InfoMapping: Erc20InfoMapping;

		type DEXIncentives: DEXIncentives<Self::AccountId, CurrencyId, Balance>;

		#[pallet::constant]
		type ExtendedProvisioningBlocks: Get<Self::BlockNumber>;

		type OnLiquidityPoolUpdated: Happened<(TradingPair, Balance, Balance)>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn liquidity_pool)]
	pub type LiquidityPool<T: Config> =
		StorageMap<_, Twox64Concat, TradingPair, (Balance, Balance), ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn trading_pair_statuses)]
	pub type TradingPairStatuses<T: Config> =
		StorageMap<_, Twox64Concat, TradingPairStatus<Balance, T::BlockNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn provisioning_pool)]
	pub type ProvisioningPool<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		TradingPair,
		Twox64Concat,
		T::AccountId,
		(Balance, Balance),
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn initial_share_exchange_rates)]
	pub type InitialShareExchangeRates<T: Config> =
		StorageMap<_, Twox64Concat, TradingPair, (ExchangeRate, ExchangeRate), ValueQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		// who, currency0, contribute0, currency1, contribute1
		AddProvision(T::AccountId, CurrencyId, Balance, CurrencyId, Balance),
		// who, currency_0, pool0, currency_1, pool1, share_increment
		AddLiquidity(T::AccountId, CurrencyId, Balance, CurrencyId, Balance, Balance),
		// who, currency_0, pool0, currency1, pool1, share_decrement
		RemoveLiquidity(T::AccountId, CurrencyId, Balance, CurrencyId, Balance, Balance),
		//trader, path, liquidity_changes
		Swap(T::AccountId, Vec<CurrencyId>, Vec<Balance>),
		//
		EnableTradingPair(TradingPair),
		ListProvisioning(TradingPair),
		DisableTradingPair(TradingPair),
		// tradingpair, pool_0, pool_1, share_amount
		ProvisioningToEnabled(TradingPair, Balance, Balance, Balance),
		//who, currency_0, contribution_0, currency_1, contribution_1
		RefundProvision(T::AccountId, CurrencyId, Balance, CurrencyId, Balance),
		//tradingpair, accumulated_provision_0, accumulated_provision_1
		ProvisioningAborted(TradingPair, Balance, Balance),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		AlreadyEnabled,
		MustBeProvisioning,
		MustBeDisabled,
		NotAllowedList,
		InvalidContributionIncrement,
		InvalidLiquidityIncrement,
		InvalidCurrencyId,
		InvalidTradingPathLength,
		InsufficientTargetAmount,
		ExcessiveSupplyAmount,
		InsufficientLiquidity,
		ZeroSupplyAmount,
		ZeroTargetAmount,
		UnacceptableShareIncrement,
		UnacceptableLiquidityWithdrawn,
		InvariantCheckFailed,
		UnqualifiedProvision,
		StillProvisioning,
		AssetUnregistered,
		InvalidTradingPath,
		NotAllowedRefund,
		CannotSwap,
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
