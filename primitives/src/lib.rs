#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency;
pub mod evm;
pub use currency::CurrencyId;


pub type BlockNumber = u32;
pub type Balance = u128;
