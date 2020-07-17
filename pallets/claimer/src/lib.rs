#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]
//! BLX claimer
//! This pallet a stakeholder claiming an ApnToken via parcel number
//! It is a WIP

use frame_support::{
	codec::{Decode, Encode}, // used for on-chain storage
	decl_event, decl_module, decl_storage, debug, decl_error, // used for all of the different macros
	dispatch::DispatchResult, // the returns from a dispatachable call which is a function that a user can call as part of an extrensic
	ensure, // used to verify things
	storage::{StorageDoubleMap, StorageMap, StorageValue}, // storage types used
	traits::Get, // no idea
};
use frame_system::{
	self as system, ensure_signed, ensure_none};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*; // imports a bunch of boiler plate

use sp_std::str; // string

/////////////////////////////////////////////////////////////////////////////////////////////////////////////

//#[cfg(test)]
//mod tests;


/////////////////////////////////////////////////////////////////////////////////////////////////
 
 
/// This is the pallet's configuration trait
pub trait Trait: balances::Trait + system::Trait {
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

/////////////////////////////////////////////////////////////////////////////////////////////////

// Custom data type
#[derive(Debug)]
enum TransactionType {
	SignedSubmitNumber,
	HttpFetching,
	None,
}

pub type GroupIndex = u32; // this is Encode (which is necessary for double_map)

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct BasinOwnerId<AccountId> {
	pub owner: AccountId,
	pub basin_id: u32,
}

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct ApnToken<
	//Hash, 
	Balance> {
	super_apn: u32,
	area: u32,
	balance: Balance,
	//annual_allocation: AnnualAllocation<Hash>, // needs to be converted to vector of structs or similar, review substrate kitties for more
}

//type ApnTokenOf<T> = ApnToken<
	//<T as system::Trait>::Hash, 
//	<T as balances::Trait>::Balance>;


#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct AnnualAllocation<Hash> {
	apn: u32,
	year: u32,
	hash: Hash, // this will be txn hash from Allocation Round
	current_years_allocation: u32,
}

type AnnualAllocationOf<T> = AnnualAllocation<<T as system::Trait>::Hash,
 //<T as balances::Trait>::Balance
 >;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_storage! {
	trait Store for Module<T: Trait> as Dmap {
		
		/// A vector of recently submitted numbers. Should be bounded
		//Numbers get(fn numbers): Vec<u64>;

		// Annual Allocation (double map to Apn Tokens and AccountId), works
		//AnnualAllocationsByApnTokenorAccount get(fn annual_allocations_by_apn_tokens_or_account):
		//	double_map hasher(blake2_128_concat) u32, hasher(blake2_128_concat) T::AccountId => AnnualAllocationOf<T>;
		
		// Get Apn Tokens for AccountId, currently returning empty struct
		//ApnTokensByAccount get(fn apn_tokens_by_account):
		//	map hasher(blake2_128_concat) T::AccountId => ApnTokenOf<T>;

		// NOT WORKING
		// Get Apn Tokens from basin_id, super_apn
		pub ApnTokensBySuperApns get(fn super_things_by_super_apns):
			map hasher(blake2_128_concat) (u32,u32) => ApnToken<T::Balance>;

		NextBasinId get (fn next_basin_id): u32;

		// Basin map
		pub Basin get(fn basin): map hasher(blake2_128_concat) u32 => BasinOwnerId<T::AccountId>;

		// wORKING
		// Balance of Apn_Tokens for owner, input is basin_id and AccountId
		pub BalanceApnTokens get(fn balance_apntokens): map hasher(blake2_128_concat) (u32, T::AccountId) => u32;

		// For membership, works
		AllMembers get(fn all_members): Vec<T::AccountId>; 
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_event! (
	pub enum Event<T>
	where
		<T as system::Trait>::Hash,
		<T as balances::Trait>::Balance,
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Event generated when a new number is accepted to contribute to the average.
		NewNumber(Option<AccountId>, u64),
		/// New member for `AllMembers` group
		NewMember(AccountId),
		/// New ApnToken claimed event includes basin_id, super_apn
		NewApnTokenClaimed(u32,u32),
		// fields of the new allocation
		//NewAnnualAllocation(u32, Hash, Balance),
		// fields of the apn_token and the annual_allocation fields
		//NewApnTokenByExistingAnnualAllocation(u32, u32, Hash, Balance),
		// ""
		NewApnTokenByNewAnnualAllocation(u32, u32, Hash, Balance),
	}
);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_error! {
	pub enum Error for Module<T: Trait> {
		// Error returned when making signed transactions in off-chain worker
		SignedSubmitNumberError,
		// Error returned when making remote http fetching
		HttpFetchingError,
		// Error returned when gh-info has already been fetched
		AlreadyFetched,
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Join the `AllMembers` vec before joining a group
		#[weight = 10_000]
		fn join_all_members(origin) -> DispatchResult {
			let new_member = ensure_signed(origin)?;
			//ensure!(!Self::is_member(&new_member), "already a member, can't join");
			<AllMembers<T>>::append(&new_member);

			Self::deposit_event(RawEvent::NewMember(new_member));
			Ok(())
		}

		// Create the digital Basin with given no parameters, lol
		#[weight = 10_000]
		fn create_basin(origin) -> DispatchResult {
			let member = ensure_signed(origin)?;

			// Generate next basin ID
			let basin_id = NextBasinId::get().checked_add(1).expect("basin id error");

			NextBasinId::put(basin_id);

			// Create new basin
			let new_basin = BasinOwnerId {
				owner: member,
				basin_id: basin_id,
			};

			// Add new basin to map
			<Basin<T>>::insert(basin_id, new_basin);

			Ok(())
		}

		// Create an ApnToken with given parameters and link to existing basin
		//
		// @param super_apn apn used as ID
		// @area area of APN related to ApnToken
		// @balance AcreFeet of water allocated to that ApnToken

		#[weight = 10_000]
		fn create_apntoken(
			origin, 
			basin_id: u32,
			super_apn: u32, 
			area: u32, 
			balance: T::Balance,
		) -> DispatchResult {
			let member = ensure_signed(origin)?;
			
			// Keeps track of how many ApnTokens any single member has, it adds 1 to the total of apntokens
			let new_balance_of_apntokens = <BalanceApnTokens<T>>::get((basin_id, member.clone())) + 1;
			// Inserts the number of ApnTokens a particular member has associated with a particular basin
			<BalanceApnTokens<T>>::insert((basin_id, member.clone()), new_balance_of_apntokens);
			
			// Create new ApnToken
			let apn_token = ApnToken {
				super_apn,
				area,
				balance, // this is balance of Acre-feet for the ApnToken
			};

			// Inserts the ApnToken on-chain, mapping to the basin id and the super_apn
			<ApnTokensBySuperApns<T>>::insert((basin_id, super_apn), apn_token);
			// Emits event
			Self::deposit_event(RawEvent::NewApnTokenClaimed(basin_id,super_apn));

			Ok(())
		}
	}
}