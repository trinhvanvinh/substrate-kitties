#![allow(clippy::from_over_into)]

use crate::evm::EvmAddress;
use crate::BlockNumber;
use bstringify::bstringify;
use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

macro_rules! create_currency_id {
    ($(#[$meta:meta])*
	$vis:vis enum TokenSymbol {
        $($(#[$vmeta:meta])* $symbol:ident($name:expr, $deci:literal) = $val:literal,)*
    }) => {
		$(#[$meta])*
		$vis enum TokenSymbol {
			$($(#[$vmeta])* $symbol = $val,)*
		}

		impl TryFrom<u8> for TokenSymbol {
			type Error = ();

			fn try_from(v: u8) -> Result<Self, Self::Error> {
				match v {
					$($val => Ok(TokenSymbol::$symbol),)*
					_ => Err(()),
				}
			}
		}

		impl Into<u8> for TokenSymbol {
			fn into(self) -> u8 {
				match self {
					$(TokenSymbol::$symbol => ($val),)*
				}
			}
		}

		impl TryFrom<Vec<u8>> for CurrencyId {
			type Error = ();
			fn try_from(v: Vec<u8>) -> Result<CurrencyId, ()> {
				match v.as_slice() {
					$(bstringify!($symbol) => Ok(CurrencyId::Token(TokenSymbol::$symbol)),)*
					_ => Err(()),
				}
			}
		}

		impl TokenInfo for CurrencyId {
			fn currency_id(&self) -> Option<u8> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some($val),)*
					_ => None,
				}
			}
			fn name(&self) -> Option<&str> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some($name),)*
					_ => None,
				}
			}
			fn symbol(&self) -> Option<&str> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some(stringify!($symbol)),)*
					_ => None,
				}
			}
			fn decimals(&self) -> Option<u8> {
				match self {
					$(CurrencyId::Token(TokenSymbol::$symbol) => Some($deci),)*
					_ => None,
				}
			}
		}

		$(pub const $symbol: CurrencyId = CurrencyId::Token(TokenSymbol::$symbol);)*

		impl TokenSymbol {
			pub fn get_info() -> Vec<(&'static str, u32)> {
				vec![
					$((stringify!($symbol), $deci),)*
				]
			}
		}

		#[test]
		#[ignore]
		fn generate_token_resources() {
			use crate::TokenSymbol::*;

			#[allow(non_snake_case)]
			#[derive(Serialize, Deserialize)]
			struct Token {
				symbol: String,
				address: EvmAddress,
			}

			// Acala tokens
			let mut acala_tokens = vec![];
			$(
				// ignore RENBTC and CASH
				if $val < 128 && $val != 20 && $val != 21 {
					acala_tokens.push(Token {
						symbol: stringify!($symbol).to_string(),
						address: EvmAddress::try_from(CurrencyId::Token(TokenSymbol::$symbol)).unwrap(),
					});
				}
			)*

			let mut acala_lp_tokens = vec![
				Token {
					symbol: "LP_ACA_AUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(AUSD), CurrencyId::Token(ACA)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_LDOT_AUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(AUSD), CurrencyId::Token(LDOT)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_LCDOT_AUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(AUSD), LCDOT).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_LCDOT_DOT".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(DOT), LCDOT).unwrap().dex_share_currency_id()).unwrap(),
				},
			];
			acala_tokens.append(&mut acala_lp_tokens);

			acala_tokens.push(Token {
				symbol: "SA_DOT".to_string(),
				address: EvmAddress::try_from(CurrencyId::StableAssetPoolToken(0)).unwrap(),
			});

			// acala_tokens.push(Token {
			// 	symbol: "SA_3USD".to_string(),
			// 	address: EvmAddress::try_from(CurrencyId::StableAssetPoolToken(1)).unwrap(),
			// });

			acala_tokens.push(Token {
				symbol: "LCDOT_13".to_string(),
				address: EvmAddress::try_from(LCDOT).unwrap(),
			});

			let mut acala_fa_tokens = vec![
				Token {
					symbol: "FA_GLMR".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(0)).unwrap(),
				},
				Token {
					symbol: "FA_PARA".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(1)).unwrap(),
				},
				Token {
					symbol: "FA_ASTR".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(2)).unwrap(),
				},
				Token {
					symbol: "FA_IBTC".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(3)).unwrap(),
				},
				Token {
					symbol: "FA_INTR".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(4)).unwrap(),
				},
				Token {
					symbol: "FA_WBTC".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(5)).unwrap(),
				},
				Token {
					symbol: "FA_WETH".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(6)).unwrap(),
				},
				Token {
					symbol: "FA_EQ".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(7)).unwrap(),
				},
				Token {
					symbol: "FA_EQD".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(8)).unwrap(),
				},
			];
			acala_tokens.append(&mut acala_fa_tokens);

			frame_support::assert_ok!(std::fs::write("../predeploy-contracts/resources/acala_tokens.json", serde_json::to_string_pretty(&acala_tokens).unwrap()));

			// Karura tokens
			let mut karura_tokens = vec![];
			$(
				if $val >= 128 {
					karura_tokens.push(Token {
						symbol: stringify!($symbol).to_string(),
						address: EvmAddress::try_from(CurrencyId::Token(TokenSymbol::$symbol)).unwrap(),
					});
				}
			)*

			let mut karura_lp_tokens = vec![
				Token {
					symbol: "LP_LKSM_KAR".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KAR), CurrencyId::Token(LKSM)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_QTZ_KAR".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KAR), CurrencyId::ForeignAsset(2)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_KAR_KSM".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KSM), CurrencyId::Token(KAR)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_LKSM_KSM".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KSM), CurrencyId::Token(LKSM)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_RMRK_KSM".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KSM), CurrencyId::ForeignAsset(0)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_ARIS_KSM".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KSM), CurrencyId::ForeignAsset(1)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_KAR_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KAR), CurrencyId::Token(KUSD)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_KSM_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KSM), CurrencyId::Token(KUSD)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_LKSM_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(LKSM), CurrencyId::Token(KUSD)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_BNC_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::Token(BNC)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_PHA_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::Token(PHA)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_KINT_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::Token(KINT)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_KBTC_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::Token(KBTC)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_RMRK_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::ForeignAsset(0)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_QTZ_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::ForeignAsset(2)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_CSM_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::ForeignAsset(5)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_AIR_KUSD".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(KUSD), CurrencyId::ForeignAsset(12)).unwrap().dex_share_currency_id()).unwrap(),
				},
				Token {
					symbol: "LP_RMRK_TAI".to_string(),
					address: EvmAddress::try_from(TradingPair::from_currency_ids(CurrencyId::Token(TAI), CurrencyId::ForeignAsset(0)).unwrap().dex_share_currency_id()).unwrap(),
				},
			];
			karura_tokens.append(&mut karura_lp_tokens);

			karura_tokens.push(Token {
				symbol: "SA_KSM".to_string(),
				address: EvmAddress::try_from(CurrencyId::StableAssetPoolToken(0)).unwrap(),
			});

			karura_tokens.push(Token {
				symbol: "SA_3USD".to_string(),
				address: EvmAddress::try_from(CurrencyId::StableAssetPoolToken(1)).unwrap(),
			});

			let mut karura_fa_tokens = vec![
				Token {
					symbol: "FA_RMRK".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(0)).unwrap(),
				},
				Token {
					symbol: "FA_ARIS".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(1)).unwrap(),
				},
				Token {
					symbol: "FA_QTZ".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(2)).unwrap(),
				},
				Token {
					symbol: "FA_MOVR".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(3)).unwrap(),
				},
				Token {
					symbol: "FA_HKO".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(4)).unwrap(),
				},
				Token {
					symbol: "FA_CSM".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(5)).unwrap(),
				},
				Token {
					symbol: "FA_KICO".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(6)).unwrap(),
				},
				Token {
					symbol: "FA_USDT".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(7)).unwrap(),
				},
				Token {
					symbol: "FA_TEER".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(8)).unwrap(),
				},
				Token {
					symbol: "FA_NEER".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(9)).unwrap(),
				},
				Token {
					symbol: "FA_KMA".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(10)).unwrap(),
				},
				Token {
					symbol: "FA_BSX".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(11)).unwrap(),
				},
				Token {
					symbol: "FA_AIR".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(12)).unwrap(),
				},
				Token {
					symbol: "FA_CRAB".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(13)).unwrap(),
				},
				Token {
					symbol: "FA_GENS".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(14)).unwrap(),
				},
				Token {
					symbol: "FA_EQD".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(15)).unwrap(),
				},
				Token {
					symbol: "FA_TUR".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(16)).unwrap(),
				},
				Token {
					symbol: "FA_PCHU".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(17)).unwrap(),
				},
				Token {
					symbol: "FA_SDN".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(18)).unwrap(),
				},
				Token {
					symbol: "FA_LT".to_string(),
					address: EvmAddress::try_from(CurrencyId::ForeignAsset(19)).unwrap(),
				},
			];
			karura_tokens.append(&mut karura_fa_tokens);

			frame_support::assert_ok!(std::fs::write("../predeploy-contracts/resources/karura_tokens.json", serde_json::to_string_pretty(&karura_tokens).unwrap()));
		}
    }
}

pub trait TokenInfo {
	fn currency_id(&self) -> Option<u8>;
	fn name(&self) -> Option<&str>;
	fn symbol(&self) -> Option<&str>;
	fn decimals(&self) -> Option<u8>;
}

create_currency_id! {
	#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, RuntimeDebug, MaxEncodedLen, Copy, PartialOrd)]
	#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
	pub enum TokenSymbol{
		ACA("Acala", 12) =0,
		AUSD("Acala Dollar", 12) = 1,
		DOT("Polkadot", 10) = 2,
	}
}

pub type ForeignAssetId = u16;
pub type Erc20Id = u32;
pub type Lease = BlockNumber;

#[derive(
	Encode, Decode, Eq, PartialEq, Clone, TypeInfo, RuntimeDebug, MaxEncodedLen, Copy, PartialOrd,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DexShare {
	Token(TokenSymbol),
	Erc20(EvmAddress),
	LiquidCrowdloan(Lease),
	ForeignAsset(ForeignAssetId),
	//StableAssetPoolToken(StableAssetPoolId)
}

#[derive(
	Encode, Decode, Eq, PartialEq, Clone, TypeInfo, RuntimeDebug, MaxEncodedLen, Copy, PartialOrd,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DexShare(DexShare, DexShare),
	Erc20(EvmAddress),
	LiquidCrowdloan(Lease),
	ForeignAsset(ForeignAssetId),
}

impl CurrencyId {
	pub fn is_token_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Token(_))
	}

	pub fn is_dex_share_currency_id(&self) -> bool {
		matches!(self, CurrencyId::DexShare(_, _))
	}

	pub fn is_erc20_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Erc20(_))
	}

	pub fn is_liquid_crowdloan_currency_id(&self) -> bool {
		matches!(self, CurrencyId::LiquidCrowdloan(_))
	}

	pub fn is_foreign_asset_currency_id(&self) -> bool {
		matches!(self, CurrencyId::ForeignAsset(_))
	}

	pub fn is_trading_pair_currency_id(&self) -> bool {
		matches!(
			self,
			CurrencyId::Token(_)
				| CurrencyId::Erc20(_)
				| CurrencyId::LiquidCrowdloan(_)
				| CurrencyId::ForeignAsset(_)
		)
	}

	// pub fn split_dex_share_currency_id(&self) -> Option<(Self, Self)> {
	// 	match self {
	// 		CurrencyId::DexShare(dex_share_0, dex_share_1) => {
	// 			let currency_id_0: CurrencyId = (*dex_share_0).into();
	// 			let currency_id_1: CurrencyId = (*dex_share_1).into();
	// 			Some((currency_id_0, currency_id_1))
	// 		},
	// 		_ => None,
	// 	}
	// }

	pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		let dex_share_0 = match currency_id_0 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			CurrencyId::LiquidCrowdloan(lease) => DexShare::LiquidCrowdloan(lease),
			CurrencyId::ForeignAsset(foreign_asset_id) => DexShare::ForeignAsset(foreign_asset_id),
			// Unsupported
			CurrencyId::DexShare(..) => return None,
		};

		let dex_share_1 = match currency_id_1 {
			CurrencyId::Token(symbol) => DexShare::Token(symbol),
			CurrencyId::Erc20(address) => DexShare::Erc20(address),
			CurrencyId::LiquidCrowdloan(lease) => DexShare::LiquidCrowdloan(lease),
			CurrencyId::ForeignAsset(foreign_asset_id) => DexShare::ForeignAsset(foreign_asset_id),

			CurrencyId::DexShare(..) => return None,
		};

		Some(CurrencyId::DexShare(dex_share_0, dex_share_1))
	}

	pub fn erc20_address(&self) -> Option<EvmAddress> {
		match self {
			CurrencyId::Erc20(address) => Some(*address),
			CurrencyId::Token(_) => EvmAddress::try_from(*self).ok(),
			_ => None,
		}
	}
}

impl From<DexShare> for u32 {
	fn from(val: DexShare) -> u32 {
		let mut bytes = [0u8; 4];
		match val {
			DexShare::Token(token) => {
				bytes[3] = token.into();
			},
			DexShare::Erc20(address) => {
				let is_zero = |&&d: &&u8| -> bool { d == 0 };
				let leading_zeros = address.as_bytes().iter().take_while(is_zero).count();
				let index = if leading_zeros > 16 { 16 } else { leading_zeros };
				bytes[..].copy_from_slice(&address[index..index + 4][..]);
			},
			DexShare::LiquidCrowdloan(lease) => {
				bytes[..].copy_from_slice(&lease.to_be_bytes());
			},
			DexShare::ForeignAsset(foreign_asset_id) => {
				bytes[2..].copy_from_slice(&foreign_asset_id.to_be_bytes());
			},
		}
		u32::from_be_bytes(bytes)
	}
}

impl Into<CurrencyId> for DexShare {
	fn into(self) -> CurrencyId {
		match self {
			DexShare::Token(token) => CurrencyId::Token(token),
			DexShare::Erc20(address) => CurrencyId::Erc20(address),
			DexShare::LiquidCrowdloan(lease) => CurrencyId::LiquidCrowdloan(lease),
			DexShare::ForeignAsset(foreign_asset_id) => CurrencyId::ForeignAsset(foreign_asset_id),
		}
	}
}

#[derive(
	Encode, Decode, Eq, PartialEq, Clone, TypeInfo, RuntimeDebug, MaxEncodedLen, Copy, PartialOrd,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyIdType {
	Token,
	DexShare,
	StableAsset,
	LiquidCrowdloan,
	ForeignAsset,
}

#[derive(
	Encode, Decode, Eq, PartialEq, Clone, TypeInfo, RuntimeDebug, MaxEncodedLen, Copy, PartialOrd,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DexShareType {
	Token,
	Erc20,
	LiquidCrowdloan,
	ForeignAsset,
	StableAssetPoolToken,
}

impl Into<DexShareType> for DexShare {
	fn into(self) -> DexShareType {
		match self {
			DexShare::Token(_) => DexShareType::Token,
			DexShare::Erc20(_) => DexShareType::Erc20,
			DexShare::LiquidCrowdloan(_) => DexShareType::LiquidCrowdloan,
			DexShare::ForeignAsset(_) => DexShareType::ForeignAsset,
		}
	}
}

pub const LCDOT: CurrencyId = CurrencyId::LiquidCrowdloan(13);

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, RuntimeDebug, Copy, PartialOrd)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AssetIds {
	Erc20(EvmAddress),
	ForeignAssetId(ForeignAssetId),
	NativeAssetId(CurrencyId),
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, TypeInfo, RuntimeDebug, PartialOrd)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetMetadata<Balance> {
	pub name: Vec<u8>,
	pub symbol: Vec<u8>,
	pub decimals: u8,
	pub minimal_balance: Balance,
}
