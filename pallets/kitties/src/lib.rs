#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

// #[cfg(test)]
// mod mock;
//
// #[cfg(test)]
// mod tests;
//
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
	traits::{tokens::ExistenceRequirement, Currency, Randomness},
	transactional,
};
use sp_runtime::{traits::Hash, RuntimeDebug};

use scale_info::TypeInfo;

use sp_io::hashing::blake2_128;

type AccountOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> =
	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub struct Kitty<T: Config> {
	pub id: [u8; 16],
	pub dna: [u8; 16],
	pub price: Option<BalanceOf<T>>,
	pub gender: Gender,
	pub owner: AccountOf<T>,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
pub enum Gender {
	Male,
	Female,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
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

		type KittyRandomness: Randomness<Self::Hash, Self::BlockNumber>;

		#[pallet::constant]
		type MaxKittyOwned: Get<u32>;

		type Currency: Currency<Self::AccountId>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn all_kitties_count)]
	pub(super) type AllKittiesCount<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_nonce)]
	pub(super) type Nonce<T: Config> = StorageValue<_, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn kitties)]
	pub(super) type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Kitty<T>>;

	#[pallet::storage]
	#[pallet::getter(fn kitties_owned)]
	pub(super) type KittiesOwned<T: Config> = StorageMap<
		_,
		Twox64Concat,
		T::AccountId,
		BoundedVec<T::Hash, T::MaxKittyOwned>,
		ValueQuery,
	>;

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub kitties: Vec<(T::AccountId, [u8; 16], Gender)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { kitties: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (acct, dna, gender) in &self.kitties {
				let _ = <Pallet<T>>::mint(acct, Some(dna.clone()), Some(gender.clone()));
			}
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

		Created(T::AccountId, T::Hash),
		PriceSet(T::AccountId, T::Hash, Option<BalanceOf<T>>),
		Transferred(T::AccountId, T::AccountId, T::Hash),
		Bought(T::AccountId, T::AccountId, T::Hash, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		KittyCountOverflow,
		ExceedMaxKittyOwned,
		BuyerIsKittyOwner,
		TransferToSelf,
		KittyNotExist,
		NotKittyOwner,
		KittyNotForSale,
		KittyBidPriceTooLow,
		NotEnoughBalance,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn create_kitty(origin: OriginFor<T>) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			let kitty_id = Self::mint(&sender.clone(), None, None)?;

			Self::deposit_event(Event::Created(sender, kitty_id));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn gender() -> Gender {
			let dna = T::KittyRandomness::random(&b"gender"[..]).0;
			if dna.as_ref()[0] % 2 == 0 {
				Gender::Male
			} else {
				Gender::Female
			}
		}

		fn gen_dna() -> [u8; 16] {
			let payload = (
				T::KittyRandomness::random(&b"dna"[..]).0,
				<frame_system::Pallet<T>>::block_number(),
			);
			payload.using_encoded(blake2_128)
		}

		fn increment_none() -> DispatchResult {
			Nonce::<T>::try_mutate(|nonce| {
				let next = nonce.checked_add(1).ok_or("Overflow")?;
				*nonce = next;
				Ok(().into())
			})
		}

		fn random_hash(sender: &T::AccountId) -> T::Hash {
			let nonce = Self::get_nonce();
			let seed = T::KittyRandomness::random_seed();

			T::Hashing::hash_of(&(seed, sender, nonce))
		}

		fn mint(
			owner: &T::AccountId,
			dna: Option<[u8; 16]>,
			gender: Option<Gender>,
		) -> Result<T::Hash, Error<T>> {
			let kitty = Kitty::<T> {
				id: dna.unwrap_or_else(Self::gen_dna),
				dna: dna.unwrap_or_else(Self::gen_dna),
				price: None,
				gender: gender.unwrap_or_else(Self::gender),
				owner: owner.clone(),
			};

			let kitty_id = T::Hashing::hash_of(&kitty);

			Ok(kitty_id)
		}
	}
}
