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

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		inherent::Vec, pallet_prelude::*, sp_runtime::traits::Hash, traits::Currency,
		traits::ExistenceRequirement, transactional,
	};
	use frame_system::pallet_prelude::*;

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct BlogPost<T: Config> {
		pub content: Vec<u8>,
		pub author: <T as frame_system::Config>::AccountId,
	}

	#[derive(Clone, Encode, Decode, PartialEq, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct BlogPostComment<T: Config> {
		pub content: Vec<u8>,
		pub author: T::AccountId,
		pub blog_post_id: T::Hash,
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		#[pallet::constant]
		type BlogPostMinBytes: Get<u32>;
		#[pallet::constant]
		type BlogPostMaxBytes: Get<u32>;
		#[pallet::constant]
		type BlogPostCommentMinBytes: Get<u32>;
		#[pallet::constant]
		type BlogPostCommentMaxBytes: Get<u32>;
	}

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn blog_posts)]
	pub(super) type BlogPosts<T: Config> = StorageMap<_, Twox64Concat, T::Hash, BlogPost<T>>;

	#[pallet::storage]
	#[pallet::getter(fn blog_post_comment)]
	pub(super) type BlogPostComments<T: Config> =
		StorageMap<_, Twox64Concat, T::Hash, Vec<BlogPostComment<T>>>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		BlogPostCreated(Vec<u8>, T::AccountId, T::Hash),
		BlogPostCommentCreated(Vec<u8>, T::AccountId, T::Hash),
		Tipped(T::AccountId, T::Hash),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,

		BlogPostNotEnoughBytes,
		BlogPostTooManyBytes,
		BlogPostCommentNotEnoughBytes,
		BlogPostCommentTooManyBytes,
		BlogPostNotFound,
		TipperIsAuthor,
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

		#[pallet::weight(10000)]
		pub fn create_blog_post(origin: OriginFor<T>, content: Vec<u8>) -> DispatchResult {
			let author = ensure_signed(origin)?;

			ensure!(
				(content.len() as u32) > T::BlogPostMinBytes::get(),
				Error::<T>::BlogPostNotEnoughBytes
			);

			ensure!(
				(content.len() as u32) < T::BlogPostMaxBytes::get(),
				Error::<T>::BlogPostTooManyBytes
			);

			let blog_post = BlogPost::<T> { content: content.clone(), author: author.clone() };
			let blog_post_id = T::Hashing::hash_of(&blog_post);

			BlogPosts::<T>::insert(blog_post_id, blog_post);

			let comments_vec: Vec<BlogPostComment<T>> = Vec::new();
			BlogPostComments::<T>::insert(blog_post_id, comments_vec);

			Self::deposit_event(Event::BlogPostCreated(content, author, blog_post_id));

			Ok(())
		}
	}
}
