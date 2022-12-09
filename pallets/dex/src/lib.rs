#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

use frame_support::traits::Currency;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
type AssetIdOf<T> = <T as Config>::AssetId;
type AssetBalanceOf<T> = <T as Config>::AssetBalance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use codec::EncodeLike;

	use frame_support::{
		pallet_prelude::*,
		sp_runtime::{
			traits::{
				AccountIdConversion, CheckedAdd, CheckedMul, CheckedSub, Convert, One, Saturating,
				Zero,
			},
			FixedPointNumber, FixedPointOperand, FixedU128,
		},
		traits::{
			fungibles::{Create, Destroy, Inspect, Mutate, Transfer},
			tokens::{Balance, WithdrawConsequence},
			ExistenceRequirement,
		},
		transactional, PalletId,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::fmt::Debug;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		type Currency: Currency<Self::AccountId>;

		type AssetBalance: Balance
			+ FixedPointOperand
			+ MaxEncodedLen
			+ MaybeSerializeDeserialize
			+ TypeInfo;

		type AssetToCurrencyBalance: Convert<Self::AssetBalance, BalanceOf<Self>>;

		type CurrencyToAssetBalance: Convert<BalanceOf<Self>, Self::AssetBalance>;

		type AssetId: MaybeSerializeDeserialize
			+ MaxEncodedLen
			+ TypeInfo
			+ Clone
			+ Debug
			+ PartialEq
			+ EncodeLike
			+ Decode;

		type Assets: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Transfer<Self::AccountId>;

		type AssetRegistry: Inspect<Self::AccountId, AssetId = Self::AssetId, Balance = Self::AssetBalance>
			+ Mutate<Self::AccountId>
			+ Create<Self::AccountId>
			+ Destroy<Self::AccountId>;

		#[pallet::constant]
		type ProviderFeeNumerator: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type ProviderFeeDenominator: Get<BalanceOf<Self>>;
		#[pallet::constant]
		type MinDeposit: Get<BalanceOf<Self>>;
	}

	pub trait ConfigHelper: Config {
		fn pallet_account() -> AccountIdOf<Self>;
		fn currency_to_asset(curr_balance: BalanceOf<Self>) -> AssetBalanceOf<Self>;
		fn asset_to_currency(asset_balance: AssetBalanceOf<Self>) -> BalanceOf<Self>;
		fn net_amount_numerator() -> BalanceOf<Self>;
	}

	impl<T: Config> ConfigHelper for T {
		fn pallet_account() -> AccountIdOf<Self> {
			Self::PalletId::get().into_account_truncating()
		}
		fn currency_to_asset(curr_balance: BalanceOf<Self>) -> AssetBalanceOf<Self> {
			Self::CurrencyToAssetBalance::convert(curr_balance)
		}
		fn asset_to_currency(asset_balance: AssetBalanceOf<Self>) -> BalanceOf<Self> {
			Self::AssetToCurrencyBalance::convert(asset_balance)
		}

		fn net_amount_numerator() -> BalanceOf<Self> {
			Self::ProviderFeeDenominator::get()
				.checked_sub(&Self::ProviderFeeNumerator::get())
				.expect("Provider fee shouldnt be greator than 100")
		}
	}

	type GenesisExchangeInfo<T> =
		(AccountIdOf<T>, AssetIdOf<T>, AssetIdOf<T>, BalanceOf<T>, AssetBalanceOf<T>);

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub exchanges: Vec<GenesisExchangeInfo<T>>,
	}

	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> GenesisConfig<T> {
			GenesisConfig { exchanges: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			let pallet_account = T::pallet_account();
			for (provider, asset_id, liquidity_token_id, currency_amount, token_amount) in
				&self.exchanges
			{
				// === create liquidity token ===
				assert!(
					!<Exchanges<T>>::contains_key(asset_id.clone()),
					"Exchange already created"
				);
				assert!(
					T::AssetRegistry::create(
						liquidity_token_id.clone(),
						pallet_account.clone(),
						false,
						<AssetBalanceOf<T>>::one()
					)
					.is_ok(),
					"Liquidity token id already in use"
				);
				// === update storage ===
				let mut exchange = Exchange {
					asset_id: asset_id.clone(),
					currency_reserve: <BalanceOf<T>>::zero(),
					token_reserve: <AssetBalanceOf<T>>::zero(),
					liquidity_token_id: liquidity_token_id.clone(),
				};

				let liquidity_minted = T::currency_to_asset(*currency_amount);

				// === currency & token transfer

				assert!(
					<T as pallet::Config>::Currency::transfer(
						&provider,
						&pallet_account,
						*currency_amount,
						ExistenceRequirement::KeepAlive
					)
					.is_ok(),
					"Provider does not have enough amount of currency"
				);

				assert!(
					T::Assets::transfer(
						asset_id.clone(),
						&provider,
						&pallet_account,
						*token_amount,
						true
					)
					.is_ok(),
					"Provider does not have enough amount of currency"
				);

				assert!(
					T::AssetRegistry::mint_into(
						liquidity_token_id.clone(),
						provider,
						liquidity_minted
					)
					.is_ok(),
					"Unexpected error while minting liquidity tokens for Provider"
				);

				// === balance update ===
				exchange.currency_reserve.saturating_accrue(*currency_amount);

				exchange.token_reserve.saturating_accrue(*token_amount);

				<Exchanges<T>>::insert(asset_id, exchange);
			}
		}
	}

	#[derive(
		Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Eq, PartialEq, RuntimeDebug, Default,
	)]
	pub struct Exchange<AssetId, Balance, AssetBalance> {
		pub asset_id: AssetId,
		pub currency_reserve: Balance,
		pub token_reserve: AssetBalance,
		pub liquidity_token_id: AssetId,
	}

	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen, Clone, Eq, PartialEq, RuntimeDebug)]
	pub enum TradeAmount<InputBalance, OutputBalance> {
		FixedInput { input_amount: InputBalance, min_output: OutputBalance },
		FixedOutput { max_input: InputBalance, output_amount: OutputBalance },
	}

	type AssetToAssetPrice<T> = (AssetBalanceOf<T>, BalanceOf<T>, AssetBalanceOf<T>);

	type ExchangeOf<T> = Exchange<AssetIdOf<T>, BalanceOf<T>, AssetBalanceOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn exchanges)]
	pub(super) type Exchanges<T: Config> =
		StorageMap<_, Twox64Concat, AssetIdOf<T>, ExchangeOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		ExchangeCreated(AssetIdOf<T>, AssetIdOf<T>),
		LiquidityAdded(
			T::AccountId,
			AssetIdOf<T>,
			BalanceOf<T>,
			AssetBalanceOf<T>,
			AssetBalanceOf<T>,
		),
		LiquidityRemoved(
			T::AccountId,
			AssetIdOf<T>,
			BalanceOf<T>,
			AssetBalanceOf<T>,
			AssetBalanceOf<T>,
		),

		CurrencyTradedForAsset(
			AssetIdOf<T>,
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			AssetBalanceOf<T>,
		),

		AssetTradeForCurrency(
			AssetIdOf<T>,
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			AssetBalanceOf<T>,
		),
	}

	#[pallet::error]
	pub enum Error<T> {
		AssetNotFound,
		ExchangeAlreadyExists,
		TokenIdTaken,
		TokenAmountIsZero,
		BalanceTooLow,
		NotEnoughTokens,
		ProviderLiquidityTooLow,
		ExchangeNotFound,
		TradeAmountIsZero,
		MaxTokenIsZero,
		CurrencyAmountIsZero,
		CurrencyAmountTooHeight,
		CurrencyAmountTooLow,
		MinLiquidityIsZero,
		MaxTokensTooLow,
		MinLiquidityTooHigh,
		LiquidityAmountIsZero,
		MinCurrencyIsZero,
		MinTokensIsZero,
		MinCurrencyTooHigh,
		MinTokensTooHigh,
		MaxCurrencyTooLow,
		MinBoughtTokensTooHigh,
		MaxSoldTokensTooLow,
		NotEnoughLiquidity,
		Overflow,
		Underflow,
		DeadlinePassed,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn create_exchange(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			liquidity_token_id: AssetIdOf<T>,
			currency_amount: BalanceOf<T>,
			token_amount: AssetBalanceOf<T>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			ensure!(currency_amount >= T::MinDeposit::get(), Error::<T>::CurrencyAmountTooLow);
			ensure!(token_amount > Zero::zero(), Error::<T>::TokenAmountIsZero);
			if T::Assets::total_issuance(asset_id.clone()).is_zero() {
				Err(Error::<T>::AssetNotFound)?
			}
			if <Exchanges<T>>::contains_key(asset_id.clone()) {
				Err(Error::<T>::ExchangeAlreadyExists)?
			}
			//=== create liquidity token ===
			T::AssetRegistry::create(
				liquidity_token_id.clone(),
				T::pallet_account(),
				false,
				<AssetBalanceOf<T>>::one(),
			)
			.map_err(|_| Error::<T>::TokenIdTaken);

			//=== update storage ===
			let exchange = Exchange {
				asset_id: asset_id.clone(),
				currency_reserve: <BalanceOf<T>>::zero(),
				token_reserve: <AssetBalanceOf<T>>::zero(),
				liquidity_token_id: liquidity_token_id.clone(),
			};

			let liquidity_minted = T::currency_to_asset(currency_amount);
			Self::do_add_liquidity(
				exchange,
				currency_amount,
				token_amount,
				liquidity_minted,
				caller,
			);

			// emit event
			Self::deposit_event(Event::ExchangeCreated(asset_id, liquidity_token_id));

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T>,
			currency_amount: BalanceOf<T>,
			min_liquidity: AssetBalanceOf<T>,
			max_tokens: AssetBalanceOf<T>,
			deadline: T::BlockNumber,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			//Self

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn get_exchange(asset_id: AssetIdOf<T>) -> Result<ExchangeOf<T>, Error<T>> {
			<Exchanges<T>>::get(asset_id).ok_or(Error::<T>::ExchangeNotFound)
		}
		fn check_deadline(deadline: T::BlockNumber) -> Result<(), Error<T>> {
			ensure!(deadline >= <frame_system::Pallet<T>>::block_number(), Error::DeadlinePassed);
			Ok(())
		}

		fn check_trade_amount<A: Zero, B: Zero>(amount: TradeAmount<A, B>) -> Result<(), Error<T>> {
			match amount {
				TradeAmount::FixedInput { input_amount, min_output } => {
					ensure!(!input_amount.is_zero(), Error::TradeAmountIsZero);
					ensure!(!min_output.is_zero(), Error::TradeAmountIsZero);
				},
				TradeAmount::FixedOutput { max_input, output_amount } => {
					ensure!(!max_input.is_zero(), Error::TradeAmountIsZero);
					ensure!(!output_amount.is_zero(), Error::TradeAmountIsZero);
				},
			}

			Ok(())
		}

		fn check_enough_currency(
			account_id: &AccountIdOf<T>,
			amount: BalanceOf<T>,
		) -> Result<(), Error<T>> {
			ensure!(
				<T as Config>::Currency::free_balance(account_id) >= amount,
				Error::<T>::BalanceTooLow
			);
			Ok(())
		}

		fn check_enough_tokens(
			asset_id: AssetIdOf<T>,
			account_id: &AccountIdOf<T>,
			amount: AssetBalanceOf<T>,
		) -> Result<(), Error<T>> {
			match T::Assets::can_withdraw(asset_id, account_id, amount) {
				WithdrawConsequence::Success => Ok(()),
				WithdrawConsequence::ReducedToZero(_) => Ok(()),
				WithdrawConsequence::UnknownAsset => Err(Error::<T>::AssetNotFound),
				_ => Err(Error::<T>::NotEnoughTokens),
			}
		}

		fn check_enough_liquidity_owned(
			exchange: ExchangeOf<T>,
			account_id: &AccountIdOf<T>,
			amount: AssetBalanceOf<T>,
		) -> Result<(), Error<T>> {
			let asset_id = exchange.liquidity_token_id;
			match T::AssetRegistry::can_withdraw(asset_id, account_id, amount) {
				WithdrawConsequence::Success => Ok(()),
				WithdrawConsequence::ReducedToZero(_) => Ok(()),
				WithdrawConsequence::UnknownAsset => Err(Error::<T>::AssetNotFound),
				_ => Err(Error::<T>::ProviderLiquidityTooLow),
			}
		}

		pub fn get_input_price(
			input_amount: BalanceOf<T>,
			input_reserve: BalanceOf<T>,
			output_reserve: &BalanceOf<T>,
		) -> Result<BalanceOf<T>, Error<T>> {
			let input_amount_with_fee = input_amount
				.checked_mul(&T::net_amount_numerator())
				.ok_or(Error::<T>::Overflow)?;

			let numerator =
				input_amount_with_fee.checked_mul(output_reserve).ok_or(Error::<T>::Overflow)?;

			let denominator = input_reserve
				.checked_mul(&T::ProviderFeeDenominator::get())
				.ok_or(Error::<T>::Overflow)?
				.checked_add(&input_amount_with_fee)
				.ok_or(Error::<T>::Overflow)?;

			Ok(numerator / denominator)
		}

		pub fn get_output_price(
			output_amount: &BalanceOf<T>,
			input_reserve: &BalanceOf<T>,
			output_reserve: &BalanceOf<T>,
		) -> Result<BalanceOf<T>, Error<T>> {
			assert!(!input_reserve.is_zero());
			assert!(!output_reserve.is_zero());
			ensure!(output_amount < output_reserve, Error::<T>::NotEnoughLiquidity);
			let numerator = input_reserve
				.checked_mul(output_amount)
				.ok_or(Error::<T>::Overflow)?
				.checked_mul(&T::ProviderFeeDenominator::get())
				.ok_or(Error::Overflow)?;
			let denominator = output_reserve
				.saturating_sub(*output_amount)
				.checked_mul(&T::net_amount_numerator())
				.ok_or(Error::<T>::Overflow)?;
			Ok((numerator / denominator).saturating_add(<BalanceOf<T>>::one()))
		}

		fn get_currency_to_asset_price(
			exchange: ExchangeOf<T>,
			amount: TradeAmount<BalanceOf<T>, AssetBalanceOf<T>>,
		) -> Result<(BalanceOf<T>, AssetBalanceOf<T>), Error<T>> {
			match amount {
				TradeAmount::FixedInput {
					input_amount: currency_amount,
					min_output: min_tokens,
				} => {
					let token_amount = Self::get_input_price(
						currency_amount,
						exchange.currency_reserve,
						&T::asset_to_currency(exchange.token_reserve),
					)?;
					let token_amount = T::currency_to_asset(token_amount);
					ensure!(token_amount >= min_tokens, Error::<T>::MinTokensTooHigh);
					Ok((currency_amount, token_amount))
				},
				TradeAmount::FixedOutput {
					max_input: max_currency,
					output_amount: token_amount,
				} => {
					let currency_amount = Self::get_output_price(
						&T::asset_to_currency(token_amount),
						&exchange.currency_reserve,
						&T::asset_to_currency(exchange.token_reserve),
					)?;
					ensure!(currency_amount <= max_currency, Error::<T>::MaxCurrencyTooLow);
					Ok((currency_amount, token_amount))
				},
			}
		}

		fn get_asset_to_currency_price(
			exchange: ExchangeOf<T>,
			amount: TradeAmount<AssetBalanceOf<T>, BalanceOf<T>>,
		) -> Result<(BalanceOf<T>, AssetBalanceOf<T>), Error<T>> {
			match amount {
				TradeAmount::FixedInput {
					input_amount: token_amount,
					min_output: min_currency,
				} => {
					let currency_amount = Self::get_input_price(
						T::asset_to_currency(token_amount),
						T::asset_to_currency(exchange.token_reserve),
						&exchange.currency_reserve,
					)?;
					ensure!(currency_amount >= min_currency, Error::<T>::MinCurrencyTooHigh);
					Ok((currency_amount, token_amount))
				},

				TradeAmount::FixedOutput {
					max_input: max_tokens,
					output_amount: currency_amount,
				} => {
					let token_amount = Self::get_output_price(
						&currency_amount,
						&T::asset_to_currency(exchange.token_reserve),
						&exchange.currency_reserve,
					)?;
					let token_amount = T::currency_to_asset(token_amount);
					ensure!(token_amount <= max_tokens, Error::<T>::MaxTokensTooLow);
					Ok((currency_amount, token_amount))
				},
			}
		}

		fn get_asset_to_asset_price(
			sold_asset_exchange: ExchangeOf<T>,
			bought_asset_exchange: ExchangeOf<T>,
			amount: TradeAmount<AssetBalanceOf<T>, AssetBalanceOf<T>>,
		) -> Result<AssetToAssetPrice<T>, Error<T>> {
			match amount {
				TradeAmount::FixedInput {
					input_amount: sold_token_amount,
					min_output: min_bought_tokens,
				} => {
					let currency_amount = Self::get_input_price(
						T::asset_to_currency(sold_token_amount),
						T::asset_to_currency(sold_asset_exchange.token_reserve),
						&sold_asset_exchange.currency_reserve,
					)?;

					let bought_token_amount = Self::get_input_price(
						currency_amount,
						bought_asset_exchange.currency_reserve,
						&T::asset_to_currency(bought_asset_exchange.token_reserve),
					)?;
					let bought_token_amount = T::currency_to_asset(bought_token_amount);
					ensure!(
						bought_token_amount >= min_bought_tokens,
						Error::<T>::MinBoughtTokensTooHigh
					);
					Ok((sold_token_amount, currency_amount, bought_token_amount))
				},

				TradeAmount::FixedOutput {
					max_input: max_sold_tokens,
					output_amount: bought_token_amount,
				} => {
					let currency_amount = Self::get_output_price(
						&T::asset_to_currency(bought_token_amount),
						&bought_asset_exchange.currency_reserve,
						&T::asset_to_currency(bought_asset_exchange.token_reserve),
					)?;
					let sold_token_amount = Self::get_output_price(
						&currency_amount,
						&T::asset_to_currency(sold_asset_exchange.token_reserve),
						&sold_asset_exchange.currency_reserve,
					)?;

					let sold_token_amount = T::currency_to_asset(sold_token_amount);
					ensure!(sold_token_amount <= max_sold_tokens, Error::<T>::MaxSoldTokensTooLow);
					Ok((sold_token_amount, currency_amount, bought_token_amount))
				},
			}
		}

		// === ===

		fn do_add_liquidity(
			mut exchange: ExchangeOf<T>,
			currency_amount: BalanceOf<T>,
			token_amount: AssetBalanceOf<T>,
			liquidity_minted: AssetBalanceOf<T>,
			provider: AccountIdOf<T>,
		) -> DispatchResult {
			// === currency & token transfer
			let asset_id = exchange.asset_id.clone();
			let pallet_account = T::pallet_account();
			<T as pallet::Config>::Currency::transfer(
				&provider,
				&pallet_account,
				currency_amount,
				ExistenceRequirement::KeepAlive,
			);
			T::Assets::transfer(asset_id.clone(), &provider, &pallet_account, token_amount, true);
			T::AssetRegistry::mint_into(
				exchange.liquidity_token_id.clone(),
				&provider,
				liquidity_minted,
			);
			// == balance update ===
			exchange.currency_reserve.saturating_accrue(currency_amount);
			exchange.token_reserve.saturating_accrue(token_amount);
			<Exchanges<T>>::insert(asset_id.clone(), exchange);

			// === emit ===
			Self::deposit_event(Event::LiquidityAdded(
				provider,
				asset_id,
				currency_amount,
				token_amount,
				liquidity_minted,
			));

			Ok(())
		}

		fn do_remove_liquidity(
			mut exchange: ExchangeOf<T>,
			currency_amount: BalanceOf<T>,
			token_amount: AssetBalanceOf<T>,
			liquidity_amount: AssetBalanceOf<T>,
			provider: AccountIdOf<T>,
		) -> DispatchResult {
			// === currency & token transfer ===
			let asset_id = exchange.asset_id.clone();
			let pallet_account = T::pallet_account();
			T::AssetRegistry::burn_from(
				exchange.liquidity_token_id.clone(),
				&provider,
				liquidity_amount,
			);
			<T as pallet::Config>::Currency::transfer(
				&pallet_account,
				&provider,
				currency_amount,
				ExistenceRequirement::AllowDeath,
			);
			T::Assets::transfer(asset_id.clone(), &pallet_account, &provider, token_amount, false);

			// === balance update ===
			exchange.currency_reserve.saturating_reduce(currency_amount);
			exchange.token_reserve.saturating_reduce(token_amount);
			<Exchanges<T>>::insert(asset_id.clone(), exchange);

			// === emit event
			Self::deposit_event(Event::LiquidityRemoved(
				provider,
				asset_id,
				currency_amount,
				token_amount,
				liquidity_amount,
			));

			Ok(())
		}

		fn swap_currency_for_asset(
			mut exchange: ExchangeOf<T>,
			currency_amount: BalanceOf<T>,
			token_amount: AssetBalanceOf<T>,
			buyer: AccountIdOf<T>,
			recipient: AccountIdOf<T>,
		) -> DispatchResult {
			// === currency & token transfer
			let asset_id = exchange.asset_id.clone();
			let pallet_account = T::pallet_account();
			if buyer != pallet_account {
				<T as pallet::Config>::Currency::transfer(
					&buyer,
					&pallet_account,
					currency_amount,
					ExistenceRequirement::AllowDeath,
				);
			}

			T::Assets::transfer(asset_id.clone(), &pallet_account, &recipient, token_amount, false);

			// === balance update ===
			exchange.currency_reserve.saturating_accrue(currency_amount);
			exchange.token_reserve.saturating_accrue(token_amount);
			<Exchanges<T>>::insert(asset_id.clone(), exchange);
			//=== emit event ===
			Self::deposit_event(Event::AssetTradeForCurrency(
				asset_id,
				buyer,
				recipient,
				currency_amount,
				token_amount,
			));

			Ok(())
		}

		fn swap_asset_for_currency(
			mut exchange: ExchangeOf<T>,
			currency_amount: BalanceOf<T>,
			token_amount: AssetBalanceOf<T>,
			buyer: AccountIdOf<T>,
			recipient: AccountIdOf<T>,
		) -> DispatchResult {
			// === currency token transfer ===
			let asset_id = exchange.asset_id.clone();
			let pallet_account = T::pallet_account();
			T::Assets::transfer(asset_id.clone(), &buyer, &pallet_account, token_amount, false);
			if recipient != pallet_account {
				<T as pallet::Config>::Currency::transfer(
					&pallet_account,
					&recipient,
					currency_amount,
					ExistenceRequirement::AllowDeath,
				);
			}

			// === balance update ===
			exchange.token_reserve.saturating_accrue(token_amount);
			exchange.currency_reserve.saturating_reduce(currency_amount);
			<Exchanges<T>>::insert(asset_id.clone(), exchange);
			//=== emit event ===
			Self::deposit_event(Event::AssetTradeForCurrency(
				asset_id,
				buyer,
				recipient,
				currency_amount,
				token_amount,
			));

			Ok(())
		}

		fn swap_asset_for_asset(
			sold_asset_exchange: ExchangeOf<T>,
			bought_asset_exchange: ExchangeOf<T>,
			currency_amount: BalanceOf<T>,
			sold_token_amount: AssetBalanceOf<T>,
			bought_token_amount: AssetBalanceOf<T>,
			buyer: AccountIdOf<T>,
			recipient: AccountIdOf<T>,
		) -> DispatchResult {
			let pallet_account = T::pallet_account();
			Self::swap_asset_for_currency(
				sold_asset_exchange,
				currency_amount,
				sold_token_amount,
				buyer,
				recipient.clone(),
			);
			Self::swap_currency_for_asset(
				bought_asset_exchange,
				currency_amount,
				bought_token_amount,
				pallet_account,
				recipient,
			)
		}
	}
}
