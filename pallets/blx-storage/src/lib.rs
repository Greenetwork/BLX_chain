#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]
//! BLX Storage
//! This pallet demonstrates how to declare and store `strcuts` that contain types
//! that come from the pallet's configuration trait.

use frame_support::{
	codec::{Decode, Encode},
	decl_event, decl_module, decl_storage,
	dispatch::DispatchResult,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::RuntimeDebug;

#[cfg(test)]
mod tests;

pub trait Trait: balances::Trait + system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

//pub type GroupIndex = u32; // this is Encode (which is necessary for double_map)

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct AnnualAllocation<Hash, Balance> {
	number: u32,
	hash: Hash,
	balance: Balance,
}

type AnnualAllocationOf<T> = AnnualAllocation<<T as system::Trait>::Hash, <T as balances::Trait>::Balance>;

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct ApnToken<Hash, Balance> {
	super_number: u32,
	annual_allocation: AnnualAllocation<Hash, Balance>,
}

decl_storage! {
	trait Store for Module<T: Trait> as NestedStructs {
		AnnualAllocationsByNumbers get(fn annual_allocations_by_numbers):
			map hasher(blake2_128_concat) u32 => AnnualAllocationOf<T>;
		ApnTokensBySuperNumbers get(fn super_things_by_super_numbers):
			map hasher(blake2_128_concat) u32 => ApnToken<T::Hash, T::Balance>;
	}
}

decl_event! (
	pub enum Event<T>
	where
		<T as system::Trait>::Hash,
		<T as balances::Trait>::Balance
	{
		// fields of the new inner thing
		NewAnnualAllocation(u32, Hash, Balance),
		// fields of the super_number and the inner_thing fields
		NewApnTokenByExistingAnnualAllocation(u32, u32, Hash, Balance),
		// ""
		NewApnTokenByNewAnnualAllocation(u32, u32, Hash, Balance),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Stores an `InnerThing` struct in the storage map
		#[weight = 10_000]
		fn insert_annual_allocation(origin, number: u32, hash: T::Hash, balance: T::Balance) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let thing = AnnualAllocation {
							number,
							hash,
							balance,
						};
			<AnnualAllocationsByNumbers<T>>::insert(number, thing);
			Self::deposit_event(RawEvent::NewAnnualAllocation(number, hash, balance));
			Ok(())
		}

		/// Stores a `SuperThing` struct in the storage map using an `InnerThing` that was already
		/// stored
		#[weight = 10_000]
		fn insert_apn_token_with_existing_annual_allocation(origin, inner_number: u32, super_number: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let annual_allocation = Self::annual_allocations_by_numbers(inner_number);
			let apn_token = ApnToken {
				super_number,
				annual_allocation: annual_allocation.clone(),
			};
			<ApnTokensBySuperNumbers<T>>::insert(super_number, apn_token);
			Self::deposit_event(RawEvent::NewApnTokenByExistingAnnualAllocation(
				super_number, annual_allocation.number, annual_allocation.hash, annual_allocation.balance));
			Ok(())
		}

		/// Stores a `SuperThing` struct in the storage map using a new `InnerThing`
		#[weight = 10_000]
		fn insert_apn_token_with_new_annual_allocation(origin, inner_number: u32, hash: T::Hash, balance: T::Balance, super_number: u32) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			// construct and insert `inner_thing` first
			let annual_allocation = AnnualAllocation {
				number: inner_number,
				hash,
				balance,
			};
			// overwrites any existing `InnerThing` with `number: inner_number` by default
			<AnnualAllocationsByNumbers<T>>::insert(inner_number, annual_allocation.clone());
			Self::deposit_event(RawEvent::NewAnnualAllocation(inner_number, hash, balance));
			// now construct and insert `super_thing`
			let apn_token = ApnToken {
				super_number,
				annual_allocation,
			};
			<ApnTokensBySuperNumbers<T>>::insert(super_number, apn_token);
			Self::deposit_event(RawEvent::NewApnTokenByNewAnnualAllocation(super_number, inner_number, hash, balance));
			Ok(())
		}
	}
}
