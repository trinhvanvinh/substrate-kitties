use codec::{Codec, FullCodec, MaxEncodedLen};
use sp_runtime::{
	traits::{AtLeast32BitUnsigned, MaybeSerializeDeserialize},
	DispatchError, DispatchResult,
};
use sp_std::{
	cmp::{Eq, Ordering, PartialEq},
	fmt::Debug,
	result,
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
		to: &AccountId,
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

	fn extend_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn remove_lock(
		lock_id: LockIdentifier,
		currency_id: Self::CurrencyId,
		who: AccountId,
	) -> DispatchResult;
}

pub trait MultiReservableCurrency<AccountId>: MultiCurrency<AccountId> {
	fn can_reserve(currency_id: Self::CurrencyId, who: AccountId, value: Self::Balance) -> bool;

	fn slash_reserved(
		currency_id: Self::CurrencyId,
		who: AccountId,
		value: Self::Balance,
	) -> Self::Balance;

	fn reserved_balance(currency_id: Self::CurrencyId, who: AccountId) -> Self::Balance;

	fn reserve(
		currency_id: Self::CurrencyId,
		who: AccountId,
		value: Self::Balance,
	) -> DispatchResult;

	fn unreserve(
		currency_id: Self::CurrencyId,
		who: AccountId,
		value: Self::Balance,
	) -> Self::Balance;

	fn repatriable_reserved(
		currency_id: Self::CurrencyId,
		slashed: AccountId,
		beneficiary: AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError>;
}

pub trait NameMultiReservableCurrency<AccountId>: MultiReservableCurrency<AccountId> {
	type ReserveIdentifier;

	fn slash_reserved_named(
		id: Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: AccountId,
		value: Self::Balance,
	) -> Self::Balance;

	fn reserved_balance_named(
		id: Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: AccountId,
	) -> Self::Balance;

	fn reserve_named(
		id: Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: AccountId,
		value: Self::Balance,
	) -> DispatchResult;

	fn unreserve_named(
		id: Self::ReserveIdentifier,
		currency_id: Self::CurrencyId,
		who: AccountId,
		value: Self::Balance,
	) -> Self::Balance;
}

pub trait BasicCurrency<AccountId> {
	type Balance: AtLeast32BitUnsigned
		+ FullCodec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ MaxEncodedLen;

	fn minimum_balance() -> Self::Balance;

	fn total_issuance() -> Self::Balance;

	fn total_balance(who: AccountId) -> Self::Balance;

	fn free_balance(who: AccountId) -> Self::Balance;

	fn ensure_can_withdraw(who: AccountId, amount: Self::Balance) -> DispatchResult;

	fn transfer(from: AccountId, to: AccountId, amount: Self::Balance) -> DispatchResult;

	fn deposit(who: AccountId, amount: Self::Balance) -> DispatchResult;

	fn withdraw(who: AccountId, amount: Self::Balance) -> DispatchResult;

	fn can_slash(who: AccountId, value: Self::Balance) -> bool;

	fn slash(who: AccountId, amount: Self::Balance) -> Self::Balance;
}

pub trait BasicCurrencyExtended<AccountId>: BasicCurrency<AccountId> {
	type Amount: arithmetic::Signed
		+ TryInto<Self::Balance>
		+ TryFrom<Self::Balance>
		+ arithmetic::SimpleArithmetic
		+ Codec
		+ Copy
		+ MaybeSerializeDeserialize
		+ Debug
		+ Default
		+ MaxEncodedLen;

	fn update_balance(who: AccountId, by_amount: Self::Amount) -> DispatchResult;
}

pub trait BasicLockableCurrency<AccountId>: BasicCurrency<AccountId> {
	type Moment;

	fn set_lock(lock_id: LockIdentifier, who: AccountId, amount: Self::Balance) -> DispatchResult;

	fn extend_lock(
		lock_id: LockIdentifier,
		who: AccountId,
		amount: Self::Balance,
	) -> DispatchResult;

	fn remove_lock(lock_id: LockIdentifier, who: AccountId) -> DispatchResult;
}

pub trait BasicReservableCurrency<AccountId>: BasicCurrency<AccountId> {
	fn can_reserve(who: AccountId, value: Self::Balance) -> bool;

	fn slash_reserved(who: AccountId, value: Self::Balance) -> Self::Balance;

	fn reserved_balance(who: AccountId) -> Self::Balance;

	fn reserve(who: AccountId, value: Self::Balance) -> DispatchResult;

	fn unreserve(who: AccountId, value: Self::Balance) -> Self::Balance;

	fn repatriable_reserved(
		slashed: AccountId,
		benificiary: AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> result::Result<Self::Balance, DispatchError>;
}

pub trait NamedBasicReservableCurrency<AccountId, ReserveIdentifier>:
	BasicReservableCurrency<AccountId>
{
	fn slash_reserved_named(
		id: ReserveIdentifier,
		who: AccountId,
		value: Self::Balance,
	) -> Self::Balance;

	fn reserved_balance_named(id: &ReserveIdentifier, who: &AccountId) -> Self::Balance;

	fn reserved_named(
		id: &ReserveIdentifier,
		who: &AccountId,
		value: Self::Balance,
	) -> DispatchResult;

	fn unresved_named(
		id: &ReserveIdentifier,
		who: &AccountId,
		value: Self::Balance,
	) -> Self::Balance;

	fn repatriable_reserved_name(
		id: ReserveIdentifier,
		slashed: AccountId,
		beneficiary: AccountId,
		value: Self::Balance,
		status: BalanceStatus,
	) -> Result<Self::Balance, DispatchError>;

	fn ensure_reserved_named(
		id: ReserveIdentifier,
		who: AccountId,
		value: Self::Balance,
	) -> DispatchResult {
		let current = Self::reserved_balance_named(&id, &who);
		match current.cmp(&value) {
			Ordering::Less => {
				Self::reserved_named(&id, &who, value.defensive_saturating_sub(current))
			},
			Ordering::Equal => Ok(()),
			Ordering::Greater => {
				Self::unresved_named(&id, &who, current.defensive_saturating_sub(value));
				Ok(())
			},
		}
	}

	fn unreserve_all_named(id: ReserveIdentifier, who: AccountId) -> Self::Balance {
		let value = Self::reserved_balance_named(&id, &who);
		//Self::unreserve_named(id, who, value);
		value
	}
}

pub trait OnDust<AccountId, CurrencyId, Balance> {
	fn on_dust(who: AccountId, currency_id: CurrencyId, amount: Balance);
}

impl<AccountId, CurrencyId, Balance> OnDust<AccountId, CurrencyId, Balance> for () {
	fn on_dust(_: AccountId, _: CurrencyId, _: Balance) {}
}
