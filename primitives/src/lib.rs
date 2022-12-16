#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub mod currency;
pub mod evm;
pub use currency::CurrencyId;

pub type BlockNumber = u32;
pub type Balance = u128;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, RuntimeDebug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TradingPair(CurrencyId, CurrencyId);
