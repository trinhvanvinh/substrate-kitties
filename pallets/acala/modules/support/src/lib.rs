#![cfg_attr(not(feature = "std"), no_std)]

pub mod evm;
pub use evm::Erc20InfoMapping;

pub mod incentives;
pub use incentives::DEXIncentives;
