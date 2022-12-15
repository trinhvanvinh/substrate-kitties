#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::FixedU128;

pub mod evm;
pub use evm::Erc20InfoMapping;

pub mod incentives;
pub use incentives::DEXIncentives;

pub type ExchangeRate = FixedU128;
