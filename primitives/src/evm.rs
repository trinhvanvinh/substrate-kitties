use sp_core::{H160, H256, U256};

use crate::currency::CurrencyId;

pub type EvmAddress = sp_core::H160;

impl TryFrom<CurrencyId> for EvmAddress {
	type Error = ();

	fn try_from(val: CurrencyId) -> Result<Self, Self::Error> {
		let mut address = [0u8; 20];
		Ok(EvmAddress::from_slice(&address))
	}
}
