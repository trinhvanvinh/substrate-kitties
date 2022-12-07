#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::TypeInfo;
use log;

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub type GameId = u64;
pub type CellIndex = u8;

#[derive(Encode, Decode, Eq, PartialEq, Clone, Debug, TypeInfo)]
pub enum Line {
	Column,
	Row,
	DownHill,
	UpHill,
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
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

	#[pallet::storage]
	#[pallet::getter(fn nextid)]
	pub type NextId<T: Config> = StorageValue<_, GameId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn board)]
	pub type Board<T: Config> = StorageDoubleMap<
		_,
		Twox64Concat,
		GameId,
		Twox64Concat,
		CellIndex,
		Option<T::AccountId>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn players)]
	pub type Players<T: Config> =
		StorageMap<_, Twox64Concat, GameId, Vec<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn turn)]
	pub type Turn<T> = StorageMap<_, Twox64Concat, GameId, u64, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn winner)]
	pub type Winner<T: Config> = StorageMap<_, Twox64Concat, GameId, Option<T::AccountId>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		NewGame(GameId, T::AccountId, T::AccountId),
		TurnTaken(GameId, T::AccountId, CellIndex),
		Win(GameId, T::AccountId),
		Draw(GameId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		NoSuchGame,
		NotYourTurn,
		CellTaken,
		NoPlayWithYourSelf,
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

		// /// An example dispatchable that may throw a custom error.
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

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn create_game(origin: OriginFor<T>, opponent: T::AccountId) -> DispatchResult {
			let challenger = ensure_signed(origin)?;
			// confirm challenger and opponent are different people
			ensure!(challenger != opponent, Error::<T>::NoPlayWithYourSelf);
			// get the next game id and update counter in storage
			let game = Self::nextid();
			NextId::<T>::put(game.wrapping_add(1));
			// store the players
			Players::<T>::insert(game, vec![challenger.clone(), opponent.clone()]);

			Self::deposit_event(Event::NewGame(game, challenger, opponent));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn take_normal_turn(
			origin: OriginFor<T>,
			game: GameId,
			cell: CellIndex,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			Self::take_turn(caller, game, cell)
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn take_winning_turn(
			origin: OriginFor<T>,
			game: GameId,
			cell: CellIndex,
			location: Line,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let turn_result = Self::take_turn(caller.clone(), game, cell);
			match turn_result {
				Err(e) => Err(e),
				Ok(()) => {
					let actually_won = match location {
						Line::Column => Self::check_vertical(game, caller.clone(), cell),
						Line::Row => Self::check_horizontal(game, caller.clone(), cell),
						Line::DownHill => Self::check_downhill(game, caller.clone()),
						Line::UpHill => Self::check_uphill(game, caller.clone()),
					};

					if actually_won {
						Winner::<T>::insert(game, Some(caller.clone()));

						Self::deposit_event(Event::Win(game, caller));

						Board::<T>::remove_prefix(game, Some(1));
						Players::<T>::remove(game);
						Turn::<T>::remove(game);
					}

					Ok(())
				},
			}
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_next_id() -> u64 {
			Self::nextid()
		}

		fn take_turn(caller: T::AccountId, game: GameId, cell: CellIndex) -> DispatchResult {
			// verify the game id
			ensure!(Players::<T>::contains_key(game), Error::<T>::NoSuchGame);

			// verify the player
			let current_turn = Self::turn(game);
			let player_index = (current_turn % 2) as usize;
			let player = Self::players(game)[player_index].clone();
			//assert_eq!(player_index, 0);
			//assert_eq!(Self::players(game).len(), 1);

			println!(
				"AAA {} {}{}{} {}",
				Self::players(game)[0],
				Self::players(game)[1],
				caller,
				player,
				current_turn
			);
			//assert_eq!(Self::players(game)[1], caller);
			ensure!(caller == player, Error::<T>::NotYourTurn);

			// verify the cell
			ensure!(!Board::<T>::contains_key(game, cell), Error::<T>::CellTaken);
			// write to the cell
			Board::<T>::insert(game, cell, Some(caller.clone()));
			// update turn counter
			Turn::<T>::insert(game, current_turn);

			Self::deposit_event(Event::TurnTaken(game, caller, cell));

			Ok(())
		}

		fn check_vertical(game: GameId, winner: T::AccountId, col: u8) -> bool {
			let size = 3;
			let cells = (0..size).map(|i| size * i + col);
			Self::player_occupies_all_cells(game, winner, cells)
		}

		fn check_horizontal(game: GameId, winner: T::AccountId, row: u8) -> bool {
			let size = 3;
			let cells = (0..size).map(|i| row * size + i);
			Self::player_occupies_all_cells(game, winner, cells)
		}

		fn check_uphill(game: GameId, winner: T::AccountId) -> bool {
			let size = 3;
			let cells = (0..size).map(|i| (size - 1) * (i + 1));
			Self::player_occupies_all_cells(game, winner, cells)
		}

		fn check_downhill(game: GameId, winner: T::AccountId) -> bool {
			let size = 3;
			let cells = (0..size).map(|i| i * (size + 1));
			Self::player_occupies_all_cells(game, winner, cells)
		}

		fn player_occupies_all_cells(
			game: GameId,
			player: T::AccountId,
			cells: impl Iterator<Item = CellIndex>,
		) -> bool {
			for cell in cells {
				match Board::<T>::get(game, cell) {
					None => return false,
					Some(p) => {
						if p != player {
							return false;
						}
					},
				}
			}
			true
		}
	}
}
