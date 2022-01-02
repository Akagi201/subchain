#![cfg(test)]
// use crate::{
// 	mock::*, pallet::{Error}
// };
use super::*;
use crate::mock::{
	new_test_ext, Event as TestEvent, Origin, SubstrateKitties, System, Test,
};

use frame_support::{
	assert_ok,
	assert_noop,
	assert_err,
};

#[test]
fn should_build_genesis_kitties() {
	new_test_ext().execute_with(|| {
		// Check we have 2 kitties, as specified
		assert_eq!(SubstrateKitties::kitty_cnt(), 2u32.into());

		// Check owners own the correct amount of kitties
		let kitties_owned_by_1 = SubstrateKitties::kitties_owned(1);
		assert_eq!(kitties_owned_by_1.len(), 1);

		let kitties_owned_by_2 = SubstrateKitties::kitties_owned(2);
		assert_eq!(kitties_owned_by_2.len(), 1);

		// Check that kitties are owned correctly
		let kid1 = kitties_owned_by_1[0];
		let kitty1 = SubstrateKitties::kitties(kid1)
			.expect("Could have this kitty ID owned by acct 1");
		assert_eq!(kitty1.owner, 1);

		let kid2 = kitties_owned_by_2[0];
		let kitty2 = SubstrateKitties::kitties(kid2)
			.expect("Could have this kitty ID owned by acct 2");
		assert_eq!(kitty2.owner, 2);
	});
}

#[test]
fn create_kitty_should_work() {
	new_test_ext().execute_with(|| {
		// create a kitty with account #1.
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(1)));
		assert_has_event!(Event::<Test>::Created(1, 3));

		// check that 3 kitties exists (together with the two from genesis)
		assert_eq!(SubstrateKitties::kitty_cnt(), 3u32.into());

		// check that account #1 owns 2 kitty
		assert_eq!(SubstrateKitties::kitties_owned(1).len(), 2);

		// check that some random account #5 does not own a kitty
		assert_eq!(SubstrateKitties::kitties_owned(5).len(), 0);

		// check that this kitty is specifically owned by account #1
		let hash = SubstrateKitties::kitties_owned(1)[1];
		let kitty = SubstrateKitties::kitties(hash).expect("should found the kitty");
		assert_eq!(kitty.owner, 1);
		assert_eq!(kitty.price, None);
	});
}

#[test]
fn create_kitty_not_enough_balance_should_fail() {
	new_test_ext().execute_with(|| {
		// create a kitty with account #3.
		assert_noop!(
			SubstrateKitties::create_kitty(Origin::signed(3)),
			Error::<Test>::InvalidReserveAmount
		);
	});
}

#[test]
fn create_kitty_count_overflow_should_fail() {
	new_test_ext().execute_with(|| {
		KittyCnt::<Test>::put(u32::max_value());
		// create a kitty with account #1.
		assert_err!(SubstrateKitties::create_kitty(Origin::signed(1)), Error::<Test>::KittyCntOverflow);
		assert_noop!(
			SubstrateKitties::create_kitty(Origin::signed(1)),
			Error::<Test>::KittyCntOverflow
		);
	});
}

#[test]
fn transfer_kitty_should_work() {
	new_test_ext().execute_with(|| {
		let kitty_id = SubstrateKitties::kitties_owned(1)[0];

		// acct 1 send kitty to acct 3
		assert_ok!(SubstrateKitties::transfer(Origin::signed(1), 3, kitty_id));
		assert_has_event!(Event::<Test>::Transferred(1, 3, kitty_id));

		// acct 1 now has 0 kitty
		assert_eq!(SubstrateKitties::kitties_owned(1).len(), 0);
		// but acct 3 does
		assert_eq!(SubstrateKitties::kitties_owned(3).len(), 1);
		let new_id = SubstrateKitties::kitties_owned(3)[0];
		// and it has the same hash
		assert_eq!(kitty_id, new_id);
	});
}

#[test]
fn transfer_non_owned_kitty_should_fail() {
	new_test_ext().execute_with(|| {
		let hash = SubstrateKitties::kitties_owned(1)[0];

		// account 0 cannot transfer a kitty with this hash.
		assert_noop!(
			SubstrateKitties::transfer(Origin::signed(9), 2, hash),
			Error::<Test>::NotKittyOwner
		);
	});
}

#[test]
fn breed_kitty_should_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(1)));
		let kitty_id1= SubstrateKitties::kitties_owned(1)[0];
		let kitty_id2= SubstrateKitties::kitties_owned(1)[1];

		assert_ok!(SubstrateKitties::breed_kitty(Origin::signed(1), kitty_id1, kitty_id2));
		assert_eq!(SubstrateKitties::kitties_owned(1).len(), 3);
	});
}

#[test]
fn breed_kitty_same_parent_should_fail() {
	new_test_ext().execute_with(|| {
		let kitty_id= SubstrateKitties::kitties_owned(1)[0];

		assert_noop!(
			SubstrateKitties::breed_kitty(Origin::signed(1), kitty_id, kitty_id),
			Error::<Test>::BreedSameParent
		);
	});
}

#[test]
fn breed_kitty_not_exist_should_fail() {
	new_test_ext().execute_with(|| {
		assert_ok!(SubstrateKitties::create_kitty(Origin::signed(1)));
		let kitty_id1= SubstrateKitties::kitties_owned(1)[0];
		let kitty_id2= SubstrateKitties::kitties_owned(1)[1];

		assert_noop!(
			SubstrateKitties::breed_kitty(Origin::signed(2), kitty_id1, kitty_id2),
			Error::<Test>::NotKittyOwner
		);
	});
}

#[test]
fn breed_kitty_not_own_should_fail() {
	new_test_ext().execute_with(|| {
		let kitty_id1= 12345;
		let kitty_id2= 54321;

		assert_noop!(
			SubstrateKitties::breed_kitty(Origin::signed(1), kitty_id1, kitty_id2),
			Error::<Test>::KittyNotExist
		);
	});
}

#[test]
fn set_price_should_work() {
	new_test_ext().execute_with(|| {
		let kitty_id = SubstrateKitties::kitties_owned(1)[0];
		assert_ok!(SubstrateKitties::set_price(Origin::signed(1), kitty_id, Some(1)));
		assert_eq!(SubstrateKitties::kitties(kitty_id).unwrap().price, Some(1));
	});
}

#[test]
fn set_price_not_own_should_fail() {
	new_test_ext().execute_with(|| {
		let kitty_id = SubstrateKitties::kitties_owned(1)[0];
		assert_noop!(
			SubstrateKitties::set_price(Origin::signed(2), kitty_id, Some(1)),
			Error::<Test>::NotKittyOwner
		);
	});
}

#[test]
fn buy_kitty_should_work() {
	new_test_ext().execute_with(|| {
		let kitty_id = SubstrateKitties::kitties_owned(1)[0];
		assert_ok!(SubstrateKitties::set_price(Origin::signed(1), kitty_id, Some(1)));
		assert_ok!(SubstrateKitties::buy_kitty(Origin::signed(2), kitty_id, 1));
	});
}

#[test]
fn buy_kitty_low_price_should_fail() {
	new_test_ext().execute_with(|| {
		let kitty_id = SubstrateKitties::kitties_owned(1)[0];
		assert_ok!(SubstrateKitties::set_price(Origin::signed(1), kitty_id, Some(2)));
		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(2), kitty_id, 1),
			Error::<Test>::KittyBidPriceTooLow
		);
	});
}

#[test]
fn buy_kitty_not_for_sale_should_fail() {
	new_test_ext().execute_with(|| {
		let kitty_id = SubstrateKitties::kitties_owned(1)[0];
		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(2), kitty_id, 1),
			Error::<Test>::KittyNotForSale
		);
	});
}

#[test]
fn buy_kitty_not_enough_balance_should_fail() {
	new_test_ext().execute_with(|| {
		let kitty_id = SubstrateKitties::kitties_owned(1)[0];
		assert_ok!(SubstrateKitties::set_price(Origin::signed(1), kitty_id, Some(2)));
		assert_noop!(
			SubstrateKitties::buy_kitty(Origin::signed(2), kitty_id, 200_000),
			Error::<Test>::NotEnoughBalance
		);
	});
}
