#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;
// use primitivesv1::multiaddress::MultiAddress;
use sp_runtime::traits::{LookupError, Saturating, 
	//StaticLookup
};
use frame_support::{decl_module, decl_error, decl_event, decl_storage, ensure, RuntimeDebug, debug};
use frame_support::dispatch::DispatchResult;
use frame_support::traits::{
	Currency, ReservableCurrency, Get, EnsureOrigin, OnUnbalanced,
	WithdrawReasons, ExistenceRequirement::KeepAlive, Imbalance,
};
use frame_system::ensure_signed;
use codec::{Encode, Decode};
use claimer::StaticLookup;

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;


pub trait Config: frame_system::Config {
	// type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type AccountIndex: frame_support::Parameter + sp_runtime::traits::Member + codec::Codec + Default + sp_runtime::traits::AtLeast32Bit + Copy;
	type Lookie: StaticLookup <Target = Self::AccountId> + StaticLookup <Source = MultiAddress<Self::AccountId, Self::AccountIndex>> ;  
	type Currency: ReservableCurrency<Self::AccountId>;
}

use sp_runtime::MultiAddress;


// decl_event!(
// 	pub enum Event<T>
// 	where
// 		AccountId = <T as frame_system::Config>::AccountId,
// 	{
// 		/// The caller is a member.
// 		IstheDude(AccountId),
// 	}
// );

decl_storage! {
	trait Store for Module<T: Config> as SimpleMap {
		SimpleMap get(fn simple_map): map hasher(blake2_128_concat) T::AccountId => u32;
	}
}

decl_error! {
	pub enum Error for Module<T: Config> {
		LookupErrorororor,
	}
}

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		// fn deposit_event() = default;

		// Checks whether the caller is a member of the set of account IDs provided by the
		// MembershipSource type. Emits an event if they are, and errors if not.
		#[weight = 10_000]
		fn check_name(origin, xxx: 
			//MultiAddress::Address32([u8; 32])
			<T::Lookie as StaticLookup>::Source,
			account: <T::Lookie as StaticLookup>::Source
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			// Get find the AccountId of the 
			let target_from_name = T::Lookie::lookup(xxx)?;
			let target_from_account = T::Lookie::lookup(account)?;

			//assert_eq!(T::Lookie::lookup, Some(account));


			// match target_from_name {
			// 	target_from_account => <SimpleMap<T>>::insert(&target_from_account,5588)
			// }

			if target_from_name == target_from_account {
				<SimpleMap<T>>::insert(&target_from_account,5588)
			}

			// Self::deposit_event(RawEvent::IstheDude(caller));

            // debug::info!("this account's name is: {:?}", name);
			//ensure!(caller == name, Error::<T>::LookupErrorororor);

			Ok(())
		}

		#[weight = 0]
		fn allocate_ACFT(origin, BasinAccount: <T::Lookie as StaticLookup>::Source, APNAccount: <T::Lookie as StaticLookup>::Source, amount: BalanceOf<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let target_APNAccount = T::Lookie::lookup(APNAccount)?;
			let target_BasinAccount = T::Lookie::lookup(BasinAccount)?;

			T::Currency::transfer(&target_BasinAccount, &target_APNAccount,
				amount, KeepAlive);

			Ok(())
		}
	}
}