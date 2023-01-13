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

use frame_support::{pallet_prelude::*, transactional};
use frame_system::pallet_prelude::*;
use primitives::{Balance, CurrencyId};
use sp_runtime::traits::{Convert, Zero};
use sp_std::{marker::PhantomData, vec::Vec};
use stable_asset::traits::StableAsset as StableAssetT;
use support::{AggregatedSwapPath, DEXManager, Swap, SwapLimit};

pub type SwapPath = AggregatedSwapPath<CurrencyId>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Dex: DEXManager<Self::AccountId, Balance, CurrencyId>;
		type StableAsset: StableAssetT<
			AssetId = CurrencyId,
			AtLeast64BitUnsigned = Balance,
			Balance = Balance,
			AccountId = Self::AccountId,
			BlockNumber = Self::BlockNumber,
		>;
		type GovernanceOrigin: EnsureOrigin<<Self as frame_system::Config>::RuntimeOrigin>;
		#[pallet::constant]
		type DexSwapJointList: Get<Vec<Vec<CurrencyId>>>;
		#[pallet::constant]
		type SwapPathLimit: Get<u32>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn aggregated_swap_paths)]
	pub type AggregatedSwapPaths<T: Config> = StorageMap<
		_,
		Twox64Concat,
		(CurrencyId, CurrencyId),
		BoundedVec<SwapPath, T::SwapPathLimit>,
		OptionQuery,
	>;

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

		CannotSwap,
		InvalidPoolId,
		InvalidTokenIndex,
		InvalidSwapPath,
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
	fn check_swap_paths(paths: &[SwapPath]) -> Result<(CurrencyId, CurrencyId), DispatchError> {
		ensure!(!paths.is_empty(), Error::<T>::InvalidSwapPath);
		let mut supply_current_id: Option<CurrencyId> = None;
		let mut previous_output_currency_id: Option<CurrencyId> = None;

		for path in paths {
			match path {
				SwapPath::Dex(dex_path) => {
					let input_currency_id = dex_path.first().ok_or(Error::<T>::InvalidSwapPath)?;
					let output_currency_id = dex_path.last().ok_or(Error::<T>::InvalidSwapPath)?;
					ensure!(input_currency_id != output_currency_id, Error::<T>::InvalidSwapPath);

					if let Some(currency_id) = previous_output_currency_id {
						ensure!(currency_id == *output_currency_id, Error::<T>::InvalidSwapPath);
					}

					if supply_current_id.is_none() {
						supply_current_id = Some(*input_currency_id);
					}
					previous_output_currency_id = Some(*output_currency_id);
				},

				SwapPath::Taiga(pool_id, supply_asset_index, target_asset_index) => {
					ensure!(supply_asset_index != target_asset_index, Error::<T>::InvalidSwapPath);
					let pool_info =
						T::StableAsset::pool(*pool_id).ok_or(Error::<T>::InvalidPoolId)?;
					let input_currency_id = pool_info
						.assets
						.get(*supply_asset_index as usize)
						.ok_or(Error::<T>::InvalidTokenIndex)?;
					let output_currency_id = pool_info
						.assets
						.get(*target_asset_index as usize)
						.ok_or(Error::<T>::InvalidTokenIndex)?;
					if let Some(currency_id) = previous_output_currency_id {
						ensure!(currency_id == *input_currency_id, Error::<T>::InvalidSwapPath);
					};

					if supply_current_id.is_none() {
						supply_current_id = Some(*input_currency_id);
					}
					previous_output_currency_id = Some(*output_currency_id);
				},
			}
		}
		ensure!(
			supply_current_id.is_some() && previous_output_currency_id.is_some(),
			Error::<T>::InvalidSwapPath
		);
		Ok((
			supply_current_id.expect("already checked"),
			previous_output_currency_id.expect("already checked"),
		))
	}

	fn get_aggregated_swap_amount(
		paths: &[SwapPath],
		swap_limit: SwapLimit<Balance>,
	) -> Option<(Balance, Balance)> {
		Self::check_swap_paths(paths).ok()?;

		match swap_limit {
			SwapLimit::ExactSupply(exact_supply_amount, min_target_amount) => {
				let mut output_amount = exact_supply_amount;

				for path in paths {
					match path {
						SwapPath::Dex(dex_path) => {
							let (_, actual_target) = T::Dex::get_swap_amount(
								dex_path,
								SwapLimit::ExactSupply(output_amount, Zero::zero()),
							)?;
							output_amount = actual_target;
						},
						SwapPath::Taiga(pool_id, supply_asset_index, target_asset_index) => {
							let (_, actual_output_amount) = T::StableAsset::get_swap_output_amount(
								*pool_id,
								*supply_asset_index,
								*target_asset_index,
								output_amount,
							)
							.map(|result| (result.dx, result.dy))?;
							output_amount = actual_output_amount;
						},
					}
				}
				if output_amount >= min_target_amount {
					return Some((exact_supply_amount, output_amount));
				}
			},
			SwapLimit::ExactTarget(max_supply_amount, exact_target_amount) => {
				let mut input_amount = exact_target_amount;

				for path in paths.iter().rev() {
					match path {
						SwapPath::Dex(dex_path) => {
							let (supply_amount, _) = T::Dex::get_swap_amount(
								dex_path,
								SwapLimit::ExactTarget(Balance::max_value(), input_amount),
							)?;
							input_amount = supply_amount;
						},
						SwapPath::Taiga(pool_id, supply_asset_index, target_asset_index) => {
							let (actual_input_amount, _) = T::StableAsset::get_swap_input_amount(
								*pool_id,
								*supply_asset_index,
								*target_asset_index,
								input_amount,
							)
							.map(|result| (result.dx, result.dy))?;

							input_amount = actual_input_amount;
						},
					}
				}
				if input_amount <= max_supply_amount {
					return Self::get_aggregated_swap_amount(
						paths,
						SwapLimit::ExactSupply(input_amount, exact_target_amount),
					);
				}
			},
		}
		None
	}

	fn do_aggregated_swap(
		who: T::AccountId,
		paths: &[SwapPath],
		swap_limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		Self::check_swap_paths(paths);

		match swap_limit {
			SwapLimit::ExactSupply(exact_supply_amount, min_target_amount) => {
				let mut output_amount = exact_supply_amount;

				for path in paths {
					match path {
						SwapPath::Dex(dex_path) => {
							let (_, actual_target) = T::Dex::swap_with_specific_path(
								who.clone(),
								dex_path,
								SwapLimit::ExactSupply(output_amount, Zero::zero()),
							)?;
							output_amount = actual_target;
						},
						SwapPath::Taiga(pool_id, supply_asset_index, target_asset_index) => {
							let pool_info =
								T::StableAsset::pool(*pool_id).ok_or(Error::<T>::InvalidPoolId)?;
							let asset_length = pool_info.assets.len() as u32;

							let (_, actual_target) = T::StableAsset::swap(
								who.clone(),
								*pool_id,
								*supply_asset_index,
								*target_asset_index,
								output_amount,
								Zero::zero(),
								asset_length,
							)?;
							output_amount = actual_target;
						},
					}
				}

				ensure!(output_amount >= min_target_amount, Error::<T>::CannotSwap);
				Ok((exact_supply_amount, output_amount))
			},

			SwapLimit::ExactTarget(max_supply_amount, exact_target_amount) => {
				let (supply_amount, _) = Self::get_aggregated_swap_amount(paths, swap_limit)
					.ok_or(Error::<T>::CannotSwap)?;
				Self::do_aggregated_swap(
					who,
					paths,
					SwapLimit::ExactSupply(supply_amount, exact_target_amount),
				)
			},
		}
	}
}

pub struct DexSwap<T>(PhantomData<T>);
impl<T: Config> Swap<T::AccountId, Balance, CurrencyId> for DexSwap<T> {
	fn get_swap_amount(
		supply_current_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Option<(Balance, Balance)> {
		T::Dex::get_best_price_swap_path(
			supply_current_id,
			target_currency_id,
			limit,
			T::DexSwapJointList::get(),
		)
		.map(|(_, supply_amount, target_amount)| (supply_amount, target_amount))
	}

	fn swap(
		who: T::AccountId,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		let path = T::Dex::get_best_price_swap_path(
			supply_currency_id,
			target_currency_id,
			limit.clone(),
			T::DexSwapJointList::get(),
		)
		.ok_or(Error::<T>::CannotSwap)?.0;
		T::Dex::swap_with_specific_path(who, &path, limit)
	}

	fn swap_by_path(
		who: T::AccountId,
		swap_path: &[CurrencyId],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		T::Dex::swap_with_specific_path(who, &swap_path, limit)
	}

	fn swap_by_aggregated_path(
		who: T::AccountId,
		swap_path: &[SwapPath],
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		Err(Error::<T>::CannotSwap.into())
	}
}

pub struct TaigaSwap<T>(PhantomData<T>);
impl<T:Config> Swap<T::AccountId, Balance, CurrencyId> for TaigaSwap<T>{
	fn get_swap_amount(
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Option<(Balance, Balance)> {
		match limit {
			SwapLimit::ExactSupply(supply_amount, min_target_amount) => {
				let (pool_id, input_index, output_index, _) =
					T::StableAsset::get_best_route(supply_currency_id, target_currency_id, supply_amount)?;

				if let Some((input_amount, output_amount)) =
					T::StableAsset::get_swap_output_amount(pool_id, input_index, output_index, supply_amount)
						.map(|result| (result.dx, result.dy))
				{
					if output_amount >= min_target_amount {
						return Some((input_amount, output_amount));
					}
				}
			}
			SwapLimit::ExactTarget(max_supply_amount, target_amount) => {
				let (pool_id, input_index, output_index, _) =
					T::StableAsset::get_best_route(supply_currency_id, target_currency_id, max_supply_amount)?;

				if let Some((input_amount, _)) =
					T::StableAsset::get_swap_input_amount(pool_id, input_index, output_index, target_amount)
						.map(|result| (result.dx, result.dy))
				{
					if !input_amount.is_zero() && input_amount <= max_supply_amount {
						// actually swap by `ExactSupply` limit
						return Self::get_swap_amount(
							supply_currency_id,
							target_currency_id,
							SwapLimit::ExactSupply(input_amount, target_amount),
						);
					}
				}
			}
		};

		None
	}

	fn swap(
		who: T::AccountId,
		supply_currency_id: CurrencyId,
		target_currency_id: CurrencyId,
		limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		let (supply_amount, min_target_amount) = match limit {
			SwapLimit::ExactSupply(supply_amount, min_target_amount) => (supply_amount, min_target_amount),
			SwapLimit::ExactTarget(_, target_amount) => {
				let (supply_amount, _) = Self::get_swap_amount(supply_currency_id, target_currency_id, limit)
					.ok_or(Error::<T>::CannotSwap)?;
				(supply_amount, target_amount)
			}
		};

		let (pool_id, input_index, output_index, _) =
			T::StableAsset::get_best_route(supply_currency_id, target_currency_id, supply_amount)
				.ok_or(Error::<T>::CannotSwap)?;
		let pool_info = T::StableAsset::pool(pool_id).ok_or(Error::<T>::InvalidPoolId)?;
		let asset_length = pool_info.assets.len() as u32;

		let (actual_supply, actual_target) = T::StableAsset::swap(
			who,
			pool_id,
			input_index,
			output_index,
			supply_amount,
			min_target_amount,
			asset_length,
		)?;

		ensure!(actual_target >= min_target_amount, Error::<T>::CannotSwap);
		Ok((actual_supply, actual_target))
	}

	// TaigaSwap do not support direct dex swap.
	fn swap_by_path(
		_who: T::AccountId,
		_swap_path: &[CurrencyId],
		_limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		Err(Error::<T>::CannotSwap.into())
	}

	// TaigaSwap do not support swap by aggregated path.
	fn swap_by_aggregated_path(
		_who: T::AccountId,
		_swap_path: &[SwapPath],
		_limit: SwapLimit<Balance>,
	) -> Result<(Balance, Balance), DispatchError> {
		Err(Error::<T>::CannotSwap.into())
	}
}

pub struct AggregatedSwap<T>(PhantomData<T>);

struct AggregatedSwapParams{
	dex_result: Option<(Balance, Balance)>,
	taiga_result: Option<(Balance, Balance)>,
	aggregated_result: Option<(Balance, Balance)>,
	swap_amount: Option<(Balance, Balance)>
}

impl<T:Config> AggregatedSwap<T>{
	fn get_swap_params(supply_currency_id: CurrencyId, target_currency_id: CurrencyId, limit: SwapLimit<Balance>)-> AggregatedSwapParams{
		let mut swap_amount: Option<(Balance, Balance)> = None;

		let dex_result = DexSwap::<T>::get_swap_amount(supply_currency_id, target_currency_id, limit.clone());
		let taiga_result = TaigaSwap::<T>::get_swap_amount(supply_currency_id, target_currency_id, limit.clone());
		let aggregated_result = Pallet::<T>::aggregated_swap_paths((supply_currency_id, target_currency_id)).and_then(|paths| Pallet::<T>::get_aggregated_swap_amount(&paths, limit.clone()));

		for result in sp_std::vec![dex_result, taiga_result, aggregated_result].iter(){
			if let Some((supply_amount, target_amount)) = result{
				if let Some((candidate_supply_amount, candidate_target_amount)) = swap_amount{
					match limit{
						SwapLimit::ExactSupply(_,_)=>{
							if target_amount > &candidate_supply_amount{
								swap_amount = *result;
							}
						}
						SwapLimit::ExactTarget(_, _)=>{
							if target_amount > &candidate_target_amount{
								swap_amount = *result;
							}
						}
					}
				}else{
					swap_amount = *result;
				}
			}
		}

		AggregatedSwapParams{
			dex_result,
			taiga_result,
			aggregated_result,
			swap_amount
		}
	}
}

impl<T:Config> Swap<T::AccountId, Balance, CurrencyId> for AggregatedSwap<T>{
	fn get_swap_amount(supply_currency_id: CurrencyId, target_currency_id: CurrencyId, limit: SwapLimit<Balance>)-> Option<(Balance, Balance)>{
		Self::get_swap_params(supply_currency_id, target_currency_id, limit).swap_amount
	}

	fn swap(who: T::AccountId, supply_currency_id: CurrencyId, target_currency_id: CurrencyId, limit: SwapLimit<Balance>)-> Result<(Balance, Balance), DispatchError>{
		let AggregatedSwapParams{
			dex_result,
			taiga_result,
			aggregated_result, 
			swap_amount
		} = Self::get_swap_params(supply_currency_id, target_currency_id, limit.clone());

		if swap_amount.is_some(){
			if dex_result == swap_amount{
				return DexSwap::<T>::swap(who, supply_currency_id, target_currency_id, limit.clone() );
			}else if taiga_result == swap_amount{
				return TaigaSwap::<T>::swap(who, supply_currency_id, target_currency_id, limit.clone());
			}else if aggregated_result == swap_amount{
				let aggregated_swap_paths = Pallet::<T>::aggregated_swap_paths((supply_currency_id, target_currency_id)).ok_or(Error::<T>::CannotSwap)?;
				return Pallet::<T>::do_aggregated_swap(who, &aggregated_swap_paths, limit);
			}
		}

		Err(Error::<T>::CannotSwap.into())
	}

	fn swap_by_aggregated_path(who: T::AccountId, swap_path: &[SwapPath], limit: SwapLimit<Balance>)-> Result<(Balance, Balance), DispatchError>{
		Pallet::<T>::do_aggregated_swap(who, swap_path, limit)
	}
}
