use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(TicTacToeModule::do_something(RuntimeOrigin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(TicTacToeModule::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			TicTacToeModule::cause_error(RuntimeOrigin::signed(1)),
			Error::<Test>::NoneValue
		);
	});
}

#[test]
fn challenger_can_go_first() {
	new_test_ext().execute_with(|| {
		assert_ok!(TicTacToeModule::create_game(RuntimeOrigin::signed(1), 2));
		//assert_eq!(TicTacToeModule::get_next_id(), 1);
		//info!("aaa");
		//assert_noop!(TicTacToeModule::take_normal_turn(RuntimeOrigin::signed(2), 0, 0), "sss");
		assert_ok!(TicTacToeModule::take_normal_turn(RuntimeOrigin::signed(1), 0, 0));
		assert_ok!(TicTacToeModule::take_normal_turn(RuntimeOrigin::signed(2), 0, 1));
		//assert_ok!(TicTacToeModule::take_normal_turn(RuntimeOrigin::signed(1), 0, 3));
		//assert_ok!(TicTacToeModule::take_normal_turn(RuntimeOrigin::signed(2), 0, 4));
	});
}
