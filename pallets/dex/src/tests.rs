use crate::pallet::ConfigHelper;
use crate::{mock::*, Error, TradeAmount};
use frame_support::{
	assert_noop, assert_ok,
	traits::{fungible::Mutate, Currency},
};

#[test]
fn create_exchange() {
	new_test_ext().execute_with(|| {
		assert_ok!(PalletDexModule::create_exchange(
			RuntimeOrigin::signed(ACCOUNT_A),
			ASSET_B,
			LIQ_TOKEN_B,
			1,
			1
		));
	})
}
