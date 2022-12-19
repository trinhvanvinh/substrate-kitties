#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub mod currency;
pub mod evm;
pub use currency::{CurrencyId, Lease};

pub type BlockNumber = u32;
pub type Balance = u128;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, RuntimeDebug, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TradingPair(CurrencyId, CurrencyId);

impl TradingPair {
	pub fn from_currency_ids(currency_id_a: CurrencyId, currency_id_b: CurrencyId) -> Option<Self> {
		if currency_id_a.is_trading_pair_currency_id()
			&& currency_id_b.is_trading_pair_currency_id()
			&& currency_id_a != currency_id_b
		{
			if currency_id_a > currency_id_b {
				Some(TradingPair(currency_id_b, currency_id_a))
			} else {
				Some(TradingPair(currency_id_a, currency_id_b))
			}
		} else {
			None
		}
	}
	pub fn first(&self) -> CurrencyId {
		self.0
	}

	pub fn second(&self) -> CurrencyId {
		self.1
	}

	pub fn dex_share_currency_id(&self) -> CurrencyId {
		CurrencyId::join_dex_share_currency_id(self.first(), self.second())
			.expect("shounld not be invalid guaranteed by construction")
	}
}
