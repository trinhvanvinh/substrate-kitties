use codec::{Codec, FullCodec, MaxEncodedLen};
use sp_runtime::traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize};
use sp_std::{
	cmp::{Eq, PartialEq},
	fmt::Debug,
};

pub use frame_support::traits::{BalanceStatus, DefensiveSaturating, LockIdentifier};

use scale_info::TypeInfo;

use crate::arithmetic;
//pub use arithmetic::{num_traits::Signed, traits::SimpleArithmetic};

pub trait MultiCurrency<AccountId> {
	type CurrencyId: FullCodec
		+ Eq
		+ PartialEq
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ TypeInfo
		+ MaxEncodedLen;

	type Balance: AtLeast32BitUnsigned
		+ FullCodec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ TypeInfo
		+ MaxEncodedLen;

	// public immutables
	fn minimum_balance(currency_id: Self::CurrencyId) -> Self::Balance;

	fn total_issuance(currency_id: Self::CurrencyId) -> Self::Balance;

	fn total_balance(currency_id: Self::CurrencyId, who: AccountId) -> Self::Balance;

	fn free_balance(currency_id: Self::CurrencyId, who: AccountId) -> Self::Balance;

	fn ensure_can_withdraw(
		currency_id: Self::CurrencyId,
		who: AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	// public mutables
	fn transfer(
		currency_id: Self::CurrencyId,
		from: AccountId,
		to: AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn deposit(
		currency_id: Self::CurrencyId,
		who: AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn withdraw(
		currency_id: Self::CurrencyId,
		who: AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn can_slash(currency_id: Self::CurrencyId, who: AccountId, value: Self::Balance) -> bool;

	fn slash(currency_id: Self::CurrencyId, who: AccountId, amount: Self::Balance)
		-> Self::Balance;
}

pub trait MultiCurrencyExtended<AccountId>: MultiCurrency<AccountId> {
	type Amount: arithmetic::Signed
		+ TryInto<Self::Balance>
		+ TryFrom<Self::Balance>
		+ arithmetic::SimpleArithmetic
		+ Codec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ TypeInfo
		+ MaxEncodedLen;

	fn update_balance(
		currency_id: Self::CurrencyId,
		who: AccountId,
		by_amount: Self::Amount,
	) -> DispatchResult;
}

pub trait MultiLockableCurrency<AccountId>: MultiCurrency<AccountId> {
	type Moment;

	fn set_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	//fn extend_lock(lock_id: LockIdentifier)
}
