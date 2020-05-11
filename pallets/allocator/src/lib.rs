#![cfg_attr(not(feature = "std"), no_std)]

/// A FRAME pallet template with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs

/// Modifying to BLX NFT Allocator 

use frame_support::{decl_event, decl_module, decl_storage, 
	//decl_error,
	codec::{Decode, Encode},
	dispatch::DispatchResult,
	//ensure,
	weights::SimpleDispatchInfo,
	sp_runtime::RuntimeDebug,
	sp_runtime::sp_std::str as str,
};
use frame_system::{self as system, ensure_signed};
//use frame_support::dispatch::Vec;
use frame_support::inherent::Vec;

//use simple_json2::{ json::{ JsonObject, JsonValue }, parse_json };

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

/// The pallet's configuration trait.
pub trait Trait: balances::Trait + system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

//#[derive(Encode, Decode, Clone, Default, RuntimeDebug)] //strings are clone not copy
//pub struct AnnualAllocation<Balance, Hash> {
//	apn: Hash,
//	balance: Balance,
//	year: u32,
//	total_allocation: u32,
//	reasoning: Hash, // will change later to txn 
//}

//type AnnualAllocationOf<T> = AnnualAllocation<<T as balances::Trait>::Balance, <T as system::Trait>::Hash>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct NonFungibleToken {
//	owner: AccountId
	//apn: Vec<u8>,
	metadata: Vec<u8>,
	//annual_allocation: AnnualAllocation<Balance, Hash>,
}

pub type APN = Vec<u8>;

// This pallet's storage items.
decl_storage! {
	// It is important to update your storage name so that your pallet's
	// storage items are isolated from other pallets.
	// ---------------------------------vvvvvvvvvvvvvv
	trait Store for Module<T: Trait> as NestedStructs {
		// info on the annual allocaiton of a particular apn
		//AnnualAllocationbyapn get(fn annual_allocation_by_apn):
		//	map hasher(blake2_128_concat) T::Hash => AnnualAllocationOf<T>;
		// info on the non fungible token (digital water apn)
		//NonFungibleTokenbyapn get(fn non_fungible_token_by_apn):
		//	map hasher(blake2_128_concat) T::Hash => NonFungibleToken<T::Balance, T::Hash>;

		// the NonFungibleToken struct owned by a particular accountID of the system
		OwnedAllo get(fn allo_of_owner):
			double_map hasher(blake2_128_concat) APN, hasher(blake2_128_concat) T::AccountId => NonFungibleToken;
	}
}

// The pallet's events
decl_event!(
	pub enum Event<T>
	where
		<T as system::Trait>::Hash,
		<T as balances::Trait>::Balance
	{
		/// field of the apn and the allocation fields
		NewNonFungibleTokenByNewAllocation(Hash, Hash, Balance, u32, u32, Hash),
	}
);

// The pallet's errors
//decl_error! {
//	pub enum Error for Module<T: Trait> {
		/// The requested user has not stored a value yet
//		NoValueStored,

		/// The value cannot be incremented further because it has reached the maimum allowed value
//		MaxValueReached,
//	}
//}

// The pallet's dispatchable functions.
decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		// Initialize errors
		//type Error = Error<T>;

		// Initialize events
		fn deposit_event() = default;

		/// Stores an `AnnualAllocation` struct in the storage map
		#[weight = SimpleDispatchInfo::default()]
		fn set_annual_allocation(origin, apn: APN, metadata: Vec<u8>) -> DispatchResult {
			let user = ensure_signed(origin)?;
			//let apn_clone = apn.clone();
			//let annual_allocation = AnnualAllocation {
			//				apn: apn.clone(),
			//				balance,
			//				year,
			//				total_allocation,
			//				reasoning,
			//			};
			//<AnnualAllocationbyapn<T>>::insert(apn.clone(), annual_allocation.clone());
			//Self::deposit_event(RawEvent::NewAllocation(apn,balance,year,total_allocation, reasoning));
			// construct NonFungibleToken and insert `AnnualAllocation`
			let non_fungible_token = NonFungibleToken {
			//						apn: apn.clone(),
									metadata,
									//annual_allocation,
			};
			//<NonFungibleTokenbyapn<T>>::insert(apn.clone(), non_fungible_token);
			//Self::deposit_event(RawEvent::NewNonFungibleTokenByNewAllocation(
			//	apn.clone(), apn.clone(), balance, year, total_allocation, reasoning
			//));

			<OwnedAllo<T>>::insert(apn,user,non_fungible_token);

			Ok(())
		}
	}
}
