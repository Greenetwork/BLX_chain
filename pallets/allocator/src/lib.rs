#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]
//! BLX allocator
//! This pallet allows stakeholders to allocate watertokens via democracy
//! It is a WIP

use core::{convert::TryInto, fmt};
use frame_support::{
	//codec::{Decode, Encode}, // used for on-chain storage
	decl_event, decl_module, decl_storage, debug, decl_error, // used for all of the different macros
	dispatch::DispatchResult, // the returns from a dispatachable call which is a function that a user can call as part of an extrensic
	ensure, // used to verify things
	storage::{StorageDoubleMap, StorageMap, StorageValue}, // storage types used
	traits::Get, // no idea
};

use water_balance::WaterBalance;
use account_set::AccountSet;
use sp_std::collections::btree_set::BTreeSet;


use parity_scale_codec::{Decode, Encode};

use frame_system::{
	self as system, ensure_signed, ensure_none,
	offchain::{
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SubmitTransaction,
	},
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*; // imports a bunch of boiler plate

use sp_std::str; // string

use sp_core::crypto::KeyTypeId;
use sp_runtime::{
	offchain as rt_offchain,
	offchain::storage::StorageValueRef,
	transaction_validity::{
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
		ValidTransaction,
	},
};

// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
//   with serde(features `std`) and alt_serde(features `no_std`).
use alt_serde::{Deserialize, Deserializer};


/////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// This is the pallet's configuration trait
pub trait Trait: balances::Trait + system::Trait {
	
	// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	type WaterBalanceSource: WaterBalance<Balance = Self::Balance>;

	type MinWaterBalance: Get<BalanceOf<Self>>;
}

type BalanceOf<T> = <<T as Trait>::WaterBalanceSource as WaterBalance>::Balance;

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_event! (
	pub enum Event<T>
	where
		//Balance = <T as system::Trait>::Balance,
		Balance = BalanceOf<T>,
	{
		/// Event generated when a new number is accepted to contribute to the average.
		WaterTokenValue(Balance),
	}
);


//the trait bound `<T as Trait>::Event: std::convert::From<RawEvent<<T as pallet_balances::Trait>::Balance>>` is not satisfied
//the trait `std::convert::From<RawEvent<<T as pallet_balances::Trait>::Balance>>` is not implemented for `<T as Trait>::Event`rustc(E0277)
//lib.rs(53, 11): required by a bound in this
//lib.rs(58, 14): required by this bound in `Trait`
//error.rs(89, 29): Error originated from macro here

//the trait bound `<T as Trait>::MinWaterBalance: frame_support::traits::Get<<T as pallet_balances::Trait>::Balance>` is not satisfied
//the trait `frame_support::traits::Get<<T as pallet_balances::Trait>::Balance>` is not implemented for `<T as Trait>::MinWaterBalance`rustc(E0277)

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_error! {
	pub enum Error for Module<T: Trait> {
		NoMatch,
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Checks whether the caller is a member of the set of account IDs provided by the
		/// MembershipSource type. Emits an event if they are, and errors if not.
		#[weight = 10_000]
		fn check_balance_is_200(origin, apn: u32) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			// Get the members from the `vec-set` pallet
			// Can call method T::MemershipSource because T: Trait, and MembershipSourse is a type in Trait
			let value = T::WaterBalanceSource::findbalance(apn);

			// Check whether the caller is a member
			ensure!(value >= T::MinWaterBalance::get(origin, apn), Error::<T>::NoMatch);

			// If the previous call didn't error, then the caller is a member, so emit the event
			Self::deposit_event(RawEvent::WaterTokenValue(value));
			Ok(())
		}
	}
}