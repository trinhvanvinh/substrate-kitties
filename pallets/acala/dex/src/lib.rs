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
use frame_support::sp_runtime::FixedPointNumber;
use frame_support::{log, pallet_prelude::*, PalletId};
use frame_system::pallet_prelude::*;
use module_support::{DEXIncentives, Erc20InfoMapping, ExchangeRate, Ratio};
use module_traits::arithmetic::One;
use module_traits::arithmetic::Zero;
use module_traits::{Happened, MultiCurrency, MultiCurrencyExtended};
use primitives::{Balance, CurrencyId, TradingPair};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::sp_runtime::traits::AccountIdConversion;
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
	pub type TradingPairStatuses<T: Config> = StorageMap<
		_,
		Twox64Concat,
		TradingPair,
		TradingPairStatus<Balance, T::BlockNumber>,
		ValueQuery,
	>;

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

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub initial_listing_trading_pairs:
			Vec<(TradingPair, (Balance, Balance), (Balance, Balance), T::BlockNumber)>,
		pub initial_enabled_trading_pairs: Vec<TradingPair>,
		pub initial_added_liquidity_pools:
			Vec<(T::AccountId, Vec<(TradingPair, (Balance, Balance))>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig {
				initial_listing_trading_pairs: vec![],
				initial_enabled_trading_pairs: vec![],
				initial_added_liquidity_pools: vec![],
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.initial_listing_trading_pairs.iter().for_each(
				|(trading_pair, min_contribution, target_provision, not_before)| {
					TradingPairStatuses::<T>::insert(
						trading_pair,
						TradingPairStatus::Provisioning(ProvisioningParameters {
							min_contribution: *min_contribution,
							target_provision: *target_provision,
							accumulated_provision: Default::default(),
							not_before: *not_before,
						}),
					)
				},
			);

			self.initial_enabled_trading_pairs.iter().for_each(|trading_pair| {
				TradingPairStatuses::<T>::insert(trading_pair, TradingPairStatus::<_, _>::Enabled);
			});

			self.initial_added_liquidity_pools.iter().for_each(|(who, trading_pairs_data)| {
				trading_pairs_data.iter().for_each(
					|(trading_pair, (deposit_amount_0, deposit_amount_1))| {
						//let result = match <Pallet<T>>::trading_pair_statuses(trading_pair) {
						// TradingPairStatus::<_, _>::Enabled => <Pallet<T>>::do_add_liquidity(
						// 	who,
						// 	trading_pair.first(),
						// 	trading_pair.second(),
						// 	*deposit_amount_0,
						// 	*deposit_amount_1,
						// 	Default::default(),
						// 	false,
						// ),
						//_ => Err(Error::<T>::MustBeEnabled.into()),
						//};
						//assert!(result.is_ok(), "genesis add liquidity pool failed");
					},
				);
			});
		}
	}

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
		MustBeEnabled,
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

impl<T: Config> Pallet<T> {
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	fn try_mutate_liquidity_pool<R, E>(
		trading_pair: &TradingPair,
		f: impl FnOnce((&Balance, &Balance)) -> sp_std::result::Result<R, E>,
	) -> sp_std::result::Result<R, E> {
		LiquidityPool::<T>::try_mutate(
			trading_pair,
			|(pool_0, pool_1)| -> sp_std::result::Result<R, E> {
				let old_pool_0 = *pool_0;
				let old_pool_1 = *pool_1;
				f((pool_0, pool_1)).map(move |result| {
					if *pool_0 != old_pool_0 || *pool_1 != old_pool_1 {
						T::OnLiquidityPoolUpdated::happened(&(*trading_pair, *pool_0, *pool_1));
					}
					result
				})
			},
		)
	}

	fn do_claim_dex_share(
		who: &T::AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
	) -> DispatchResult {
		let trading_pair = TradingPair::from_currency_ids(currency_id_a, currency_id_b)
			.ok_or(Error::<T>::InvalidCurrencyId)?;

		ensure!(
			!matches!(
				Self::trading_pair_statuses(trading_pair),
				TradingPairStatus::<_, _>::Provisioning(_)
			),
			Error::<T>::StillProvisioning
		);

		ProvisioningPool::<T>::try_mutate_exists(
			trading_pair,
			who,
			|maybe_contribution| -> DispatchResult {
				if let Some((contribution_0, contribution_1)) = maybe_contribution.take() {
					let (exchange_rate_0, exchange_rate_1) =
						Self::initial_share_exchange_rates(trading_pair);
					let shares_from_provision_0 = exchange_rate_0
						.checked_mul_int(contribution_0)
						.ok_or(Error::<T>::StorageOverflow)?;
					let shares_from_provision_1 = exchange_rate_1
						.checked_mul_int(contribution_1)
						.ok_or(Error::<T>::StorageOverflow)?;

					let shares_to_claim = shares_from_provision_0
						.checked_add(shares_from_provision_1)
						.ok_or(Error::<T>::StorageOverflow)?;

					T::Currency::transfer(
						trading_pair.dex_share_currency_id(),
						&Self::account_id(),
						who,
						shares_to_claim,
					);

					// decrease ref count
					frame_system::Pallet::<T>::dec_consumers(who);
				}

				Ok(())
			},
		);

		// clear initialShareExchangeRates once it is all claimed
		if ProvisioningPool::<T>::iter_prefix(trading_pair).next().is_none() {
			InitialShareExchangeRates::<T>::remove(trading_pair);
		}

		Ok(())
	}

	fn do_add_provision(
		who: &T::AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		contribution_a: Balance,
		contribution_b: Balance,
	) -> DispatchResult {
		let trading_pair = TradingPair::from_currency_ids(currency_id_a, currency_id_b)
			.ok_or(Error::<T>::InvalidCurrencyId)?;

		let mut provision_parameters = match Self::trading_pair_statuses(trading_pair) {
			TradingPairStatus::<_, _>::Provisioning(provision_parameters) => provision_parameters,
			_ => return Err(Error::<T>::MustBeProvisioning.into()),
		};

		let (contribution_0, contribution_1) = if currency_id_a == trading_pair.first() {
			(contribution_a, contribution_b)
		} else {
			(contribution_b, contribution_a)
		};

		ensure!(
			contribution_0 >= provision_parameters.min_contribution.0
				|| contribution_1 >= provision_parameters.min_contribution.1,
			Error::<T>::InvalidContributionIncrement
		);

		ProvisioningPool::<T>::try_mutate_exists(
			trading_pair,
			&who,
			|maybe_pool| -> DispatchResult {
				let existed = maybe_pool.is_some();
				let mut pool = maybe_pool.unwrap();

				pool.0 = pool.0.checked_add(contribution_a).ok_or(Error::<T>::StorageOverflow)?;
				pool.1 = pool.1.checked_add(contribution_b).ok_or(Error::<T>::StorageOverflow)?;

				let module_account_id = Self::account_id();
				T::Currency::transfer(
					trading_pair.first(),
					who,
					&module_account_id,
					contribution_a,
				);
				T::Currency::transfer(
					trading_pair.second(),
					who,
					&module_account_id,
					contribution_b,
				);

				*maybe_pool = Some(pool);

				if !existed && maybe_pool.is_some() {
					if frame_system::Pallet::<T>::inc_consumers(&who).is_err() {
						log::warn!(
							"Warning: Attempt to introduce lock consumer reference, 
						yet no providers.This is unexpected but should be safe."
						);
					}
				}

				provision_parameters.accumulated_provision.0 = provision_parameters
					.accumulated_provision
					.0
					.checked_add(contribution_0)
					.ok_or(Error::<T>::StorageOverflow)?;

				provision_parameters.accumulated_provision.1 = provision_parameters
					.accumulated_provision
					.0
					.checked_add(contribution_1)
					.ok_or(Error::<T>::StorageOverflow)?;

				TradingPairStatuses::<T>::insert(
					trading_pair,
					TradingPairStatus::<_, _>::Provisioning(provision_parameters),
				);

				Self::deposit_event(Event::AddProvision(
					who.clone(),
					trading_pair.first(),
					contribution_0,
					trading_pair.second(),
					contribution_1,
				));

				Ok(())
			},
		)

		//Ok(())
	}

	fn do_add_liquidity(
		who: T::AccountId,
		currency_id_a: CurrencyId,
		currency_id_b: CurrencyId,
		max_amount_a: Balance,
		max_amount_b: Balance,
		min_share_increment: Balance,
		stake_increment_share: bool,
	) -> Result<(Balance, Balance, Balance), DispatchError> {
		let trading_pair = TradingPair::from_currency_ids(currency_id_a, currency_id_b)
			.ok_or(Error::<T>::InvalidCurrencyId)?;

		ensure!(
			matches!(Self::trading_pair_statuses(trading_pair), TradingPairStatus::<_, _>::Enabled),
			Error::<T>::MustBeEnabled
		);

		ensure!(
			!max_amount_a.is_zero() && !max_amount_b.is_zero(),
			Error::<T>::InvalidLiquidityIncrement
		);

		Self::try_mutate_liquidity_pool(
			&trading_pair,
			|(pool_0, pool_1)| -> Result<(Balance, Balance, Balance), DispatchError> {
				let dex_share_currency_id = trading_pair.dex_share_currency_id();
				let total_shares = T::Currency::total_issuance(dex_share_currency_id);
				let (max_amount_0, max_amount_1) = if currency_id_a == trading_pair.first() {
					(max_amount_a, max_amount_b)
				} else {
					(max_amount_b, max_amount_a)
				};

				let (pool_0_increment, pool_1_increment, share_increment): (
					Balance,
					Balance,
					Balance,
				) = if total_shares.is_zero() {
					let (exchange_rate_0, exchange_rate_1) = (
						ExchangeRate::one(),
						ExchangeRate::checked_from_rational(max_amount_0, max_amount_1)
							.ok_or(Error::<T>::StorageOverflow)?,
					);

					let shares_from_token_0 = exchange_rate_0
						.checked_mul_int(max_amount_0)
						.ok_or(Error::<T>::StorageOverflow)?;
					let shares_from_token_1 = exchange_rate_1
						.checked_mul_int(max_amount_1)
						.ok_or(Error::<T>::StorageOverflow)?;
					let initial_shares = shares_from_token_0
						.checked_add(shares_from_token_1)
						.ok_or(Error::<T>::StorageOverflow)?;

					(max_amount_0, max_amount_1, initial_shares)
				} else {
					let exchange_rate_0_1 = ExchangeRate::checked_from_rational(*pool_1, *pool_0)
						.ok_or(Error::<T>::StorageOverflow)?;
					let input_exchange_rate_0_1 =
						ExchangeRate::checked_from_rational(max_amount_1, max_amount_0)
							.ok_or(Error::<T>::StorageOverflow)?;

					if input_exchange_rate_0_1 <= exchange_rate_0_1 {
						let exchange_rate_1_0 =
							ExchangeRate::checked_from_rational(*pool_0, *pool_1)
								.ok_or(Error::<T>::StorageOverflow)?;
						let amount_0 = exchange_rate_1_0
							.checked_mul_int(max_amount_1)
							.ok_or(Error::<T>::StorageOverflow)?;
						let share_increment = Ratio::checked_from_rational(amount_0, *pool_0)
							.and_then(|n| n.checked_mul_int(total_shares))
							.ok_or(Error::<T>::StorageOverflow)?;
						(amount_0, max_amount_1, share_increment)
					} else {
						let amount_1 = exchange_rate_0_1
							.checked_mul_int(max_amount_0)
							.ok_or(Error::<T>::StorageOverflow)?;
						let share_increment = Ratio::checked_from_rational(amount_1, *pool_1)
							.and_then(|n| n.checked_mul_int(total_shares))
							.ok_or(Error::<T>::StorageOverflow)?;

						(max_amount_0, amount_1, share_increment)
					}
				};

				ensure!(
					!share_increment.is_zero()
						&& !pool_0_increment.is_zero()
						&& !pool_1_increment.is_zero(),
					Error::<T>::InvalidLiquidityIncrement
				);

				ensure!(
					share_increment >= min_share_increment,
					Error::<T>::UnacceptableShareIncrement
				);

				let module_account_id = Self::account_id();
				T::Currency::transfer(
					trading_pair.first(),
					&who,
					&module_account_id,
					pool_0_increment,
				);
				T::Currency::transfer(
					trading_pair.second(),
					&who,
					&module_account_id,
					pool_1_increment,
				);

				T::Currency::deposit(dex_share_currency_id, who.clone(), share_increment);

				//*pool_0 =
				pool_0.checked_add(pool_0_increment).ok_or(Error::<T>::StorageOverflow)?;
				//*pool_1 =
				pool_1.checked_add(pool_1_increment).ok_or(Error::<T>::StorageOverflow)?;

				if stake_increment_share {
					T::DEXIncentives::do_deposit_dex_share(
						who.clone(),
						dex_share_currency_id,
						share_increment,
					)?;
				}

				Self::deposit_event(Event::AddLiquidity(
					who,
					trading_pair.first(),
					pool_0_increment,
					trading_pair.second(),
					pool_1_increment,
					share_increment,
				));

				if currency_id_a == trading_pair.first() {
					Ok((pool_0_increment, pool_1_increment, share_increment))
				} else {
					Ok((pool_1_increment, pool_0_increment, share_increment))
				}
			},
		)
	}
}
