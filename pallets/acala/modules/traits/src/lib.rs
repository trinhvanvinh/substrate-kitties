#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub mod arithmetic;
pub mod currency;

pub use currency::{MultiCurrency, MultiCurrencyExtended};

pub trait Happened<T> {
	fn happened(t: &T);
}
