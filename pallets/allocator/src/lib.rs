#![cfg_attr(not(feature = "std"), no_std)]

// use pallet_assets as assets;

use frame_support::{
	//codec::{Decode, Encode}, // used for on-chain storage
	decl_event, decl_module, decl_storage, debug, decl_error, // used for all of the different macros
	dispatch::{DispatchResult, DispatchError},// the returns from a dispatachable call which is a function that a user can call as part of an extrensic
	ensure, // used to verify things
	// RuntimeDebug,
	storage::{StorageDoubleMap, StorageMap, StorageValue}, // storage types used
	traits::{
		Get, // no idea
		ReservableCurrency, Currency, InstanceFilter, OriginTrait, IsType, 
		EnsureOrigin, OnUnbalanced, WithdrawReasons, ExistenceRequirement::KeepAlive, Imbalance,
		//IsSubType, //cant find this?
		Vec,
	},
	Parameter,
	weights::{Weight, GetDispatchInfo},
};
use frame_system::{
	self as system, ensure_signed, ensure_none};

use sp_runtime::{
	RuntimeDebug,
	traits::{
		AtLeast32BitUnsigned, Zero, StaticLookup, Saturating, CheckedSub, CheckedAdd, Member, CheckedMul, AccountIdLookup,
	}
};

use codec::{Encode, Decode, EncodeLike, HasCompact};

use claimer::{ApnSet, LookupError};

pub use pallet_assets::WeightInfo;
use sp_runtime::MultiAddress; // might be needed idk
// use claimer::StaticLookup; // idk need to get this straightend out if the StaticLookup type in the claimer pallet is even necessary anymore
use sp_runtime::traits::Lookup;
pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
// pub type FrozenBalance<T> = <T as pallet_assets::Config>::FrozenBalance;

// pub type AssetIdOf<T> = <T as pallet_assets::Config>::AssetId;
// pub type AssetBalanceOf<T> = <T as pallet_assets::Config>::Balance;


pub trait Config: frame_system::Config { 
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The units in which we record balances.
		type Balance1: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

		/// The arithmetic type of asset identifier.
		type AssetId: Member + Parameter + Default + Copy + HasCompact;

		/// The currency mechanism.
		type Currency: ReservableCurrency<Self::AccountId>;

		/// The origin which may forcibly create or destroy an asset.
		type ForceOrigin: EnsureOrigin<Self::Origin>;

		/// The basic amount of funds that must be reserved when creating a new asset class.
		type AssetDepositBase: Get<BalanceOf<Self>>;

		/// The additional funds that must be reserved for every zombie account that an asset class
		/// supports.
		type AssetDepositPerZombie: Get<BalanceOf<Self>>;

		/// The maximum length of a name or symbol stored on-chain.
		type StringLimit: Get<u32>;

		/// The basic amount of funds that must be reserved when adding metadata to your asset.
		type MetadataDepositBase: Get<BalanceOf<Self>>;

		/// The additional funds that must be reserved for the number of bytes you store in your
		/// metadata.
		type MetadataDepositPerByte: Get<BalanceOf<Self>>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		// Might be needed idk, will need to be added to runtime/src/lib.rs
		// type AccountIndex: frame_support::Parameter + sp_runtime::traits::Member + codec::Codec + Default + sp_runtime::traits::AtLeast32Bit + Copy;
		// type Lookie: StaticLookup <Target = Self::AccountId> + StaticLookup <Source = MultiAddress<Self::AccountId, Self::AccountIndex>> ;  

		type Name: ApnSet <Name = [u8; 32]> + EncodeLike<<Self as frame_system::Config>::AccountId>;
		type Lookie: StaticLookup <Target = Self::AccountId>;  

}

decl_storage! {
	trait Store for Module<T: Config> as Asset {
		/// Next available ID for user-created asset.
		pub NextAssetId get(fn next_asset_id) : T::AssetId;

		pub Balances:
			double_map hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => T::Balance1;

		pub TotalSupply:
			map hasher(blake2_128_concat) T::AssetId => T::Balance1;

		QueueAvailable get(fn queue_available): bool;
	}
}


decl_event!(
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		<T as Config>::Balance1,
		<T as Config>::AssetId,
		// AssetOptions = AssetOptions<<T as Config>::Balance, <T as frame_system::Config>::AccountId>
	{
		/// Asset created. [asset_id, creator, asset_options]
		// Created(AssetId, AccountId, AssetOptions),
		/// Asset transfer succeeded. [asset_id, from, to, amount]
		Transferred(AssetId, AccountId, AccountId, Balance1),
		/// Asset permission updated. [asset_id, new_permissions]
		// PermissionUpdated(AssetId, PermissionLatest<AccountId>),
		/// New asset minted. [asset_id, account, amount]
		Minted(AssetId, AccountId, Balance1),
		/// Asset burned. [asset_id, account, amount]
		Burned(AssetId, AccountId, Balance1),
	}
);

decl_error! (
	pub enum Error for Module<T: Config> {
		/// The caller is not a member
		somerror,
	}
);


decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		
		#[weight = 0]
		pub fn insert_new_task(origin) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			QueueAvailable::put(true);
			Ok(())
		}

		// should be able to query all of the proxy accounts created for APN's by claimer pallet and then pass them to issue_token_airdrop
		// pub fn allocate(origin, acft: T::Balance1) -> DispatchResult {
		// 	let _sender = ensure_signed(origin)?;


		// }

		// #[weight = 0]
		// fn check_apn(origin) -> Result<Vec<<T::Name as ApnSet>::Name>, Error<T>> {
		// 	let popper = T::Name::apnsset().iter().cloned().collect();
		// 	Ok(popper)
		// }

		// #[weight = 0]
		// fn check_apn(origin) -> Vec<<T::Name as ApnSet>::Name> {
		// 	let popper = T::Name::apnsset().iter().cloned().collect();
		// 	popper
		// }

		#[weight = 0]
		pub fn issue_token_airdrop(
			origin, 
			// apn: Vec<T::AccountId>, 
			atokens: T::Balance1
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			// const ACCOUNT_ALICE: u64 = 1;
			// const ACCOUNT_BOB: u64 = 2;
			// const COUNT_AIRDROP_RECIPIENTS: u64 = 2;
			
			// let TOKENS_FIXED_SUPPLY = &atokens;
			// const TOKENS_FIXED_SUPPLY: u64 = 100;

			// ensure!(!COUNT_AIRDROP_RECIPIENTS.is_zero(), ArithmeticError::DivisionByZero);

			// let popper: Vec<<T::Name as ApnSet>::Name> = T::Name::apnsset().iter().cloned().collect();

			let friendly_names: Vec<<T::Name as ApnSet>::Name> = T::Name::apnsset().iter().cloned().collect();

			let asset_id = Self::next_asset_id();

			for i in 0..friendly_names.len() {
				let apn_name = friendly_names[i];

				let acc= AccountIdLookup::<T::AccountId, T::Index>::lookup(MultiAddress::Address32(apn_name))?;
				<Balances<T>>::insert(asset_id, acc, &atokens);

				// <Balances<T>>::insert(asset_id, AccountIdLookup::lookup(MultiAddress::Address32(apn_name)).into(), &atokens);

				// match apn_name.into() {
				// 	MultiAddress::<T::AccountId, T::Index>::Address32(hash) => {
				// 		// Lookup::<T>::get(hash).ok_or(LookupError)
				// 		// T::Lookup::lookup(apn_name.into())
				// 		// let lookup_apn = 
				// 		// let lookup_apn = T::Lookie::lookup(hash.);


						
				// 		<Balances<T>>::insert(asset_id, AccountIdLookup::lookup(MultiAddress::Address32(hash)).into(), &atokens);
				// 		// <Balances<T>>::insert(asset_id, MultiAddress::Address32(hash), &atokens);

				// 	},
				// 	_ => (),
				// }
				// let target_from_apn = T::Lookup::lookup(apn_name.into());
				// <Balances<T>>::insert(asset_id, target_from_apn, &atokens);
			}


		// #[weight = 0]
		// pub fn issue_token_airdrop(origin, apn: Vec<T::AccountId>, atokens: T::Balance1) -> DispatchResult {
		// 	let sender = ensure_signed(origin)?;

		// 	// const ACCOUNT_ALICE: u64 = 1;
		// 	// const ACCOUNT_BOB: u64 = 2;
		// 	// const COUNT_AIRDROP_RECIPIENTS: u64 = 2;
			
		// 	// let TOKENS_FIXED_SUPPLY = &atokens;
		// 	// const TOKENS_FIXED_SUPPLY: u64 = 100;

		// 	// ensure!(!COUNT_AIRDROP_RECIPIENTS.is_zero(), ArithmeticError::DivisionByZero);

		// 	let asset_id = Self::next_asset_id();

		// 	for i in 0..apn.len() {
		// 		<Balances<T>>::insert(asset_id, &apn[i], &atokens);
		// 	}

	
			// for (acc) in 
			// let targets = apn.len();

			// <NextAssetId<T>>::mutate(|asset_id| *asset_id += 1);
			// <Balances<T>>::insert(asset_id, &apn, &atokens);
			// <Balances<T>>::insert(asset_id, &ACCOUNT_BOB, TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
			// <TotalSupply<T>>::insert(asset_id, &atokens.checked_mul(targets); // spend time here figuring out how to multiply # of apn accounts by assets

			// Self::deposit_event(RawEvent::Issued(asset_id, sender, TOKENS_FIXED_SUPPLY));
			Ok(())
		}
	}
}

// impl<T:Config> Module<T> for {

// }