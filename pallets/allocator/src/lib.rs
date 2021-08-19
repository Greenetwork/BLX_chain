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
		AtLeast32BitUnsigned, Zero, StaticLookup, Saturating, CheckedSub, CheckedAdd, Member, CheckedMul,
	}
};

use codec::{Encode, Decode, HasCompact};

pub use pallet_assets::WeightInfo;
use sp_runtime::MultiAddress; // might be needed idk
// use claimer::StaticLookup; // idk need to get this straightend out if the StaticLookup type in the claimer pallet is even necessary anymore

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
		type AccountIndex: frame_support::Parameter + sp_runtime::traits::Member + codec::Codec + Default + sp_runtime::traits::AtLeast32Bit + Copy;
		type Lookie: StaticLookup <Target = Self::AccountId> + StaticLookup <Source = MultiAddress<Self::AccountId, Self::AccountIndex>> ;  
}

decl_storage! {
	trait Store for Module<T: Config> as Asset {
		/// Next available ID for user-created asset.
		pub NextAssetId get(fn next_asset_id) : T::AssetId;

		pub Balances get(fn balances):
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
		

		/// Trade from APN account to APN account, there is so much extra code in the formal substrate implementation of this (frame/assets/src/functions.rs) . relevant in the future. 
		#[weight = 0]
		pub fn trade_tokens(origin, 
			asset_id: T::AssetId, 
			fromapn: <T::Lookie as StaticLookup>::Source, 
			toapn: <T::Lookie as StaticLookup>::Source, 
			amt: T::Balance1) -> DispatchResult {
			let _sender = ensure_signed(origin)?;
			

			let target_from = T::Lookie::lookup(fromapn)?;
			let target_to = T::Lookie::lookup(toapn)?;


			let fromapn_account_balance = Self::balances(asset_id, &target_from);
			let toapn_account_balance = Self::balances(asset_id, &target_to);

			let updated_fromapn_account_balance : T::Balance1 = fromapn_account_balance.checked_sub(&amt).unwrap();
			let updated_toapn_account_balance : T::Balance1 = toapn_account_balance.checked_add(&amt).unwrap();

			<Balances<T>>::insert(asset_id, &target_from, updated_fromapn_account_balance);
			<Balances<T>>::insert(asset_id, &target_to, updated_toapn_account_balance);

			// let mut source_account = Account::<T, I>::get(asset_id, &fromapn);

			Ok(())
		}

		// }

		// Automatic airdrop dependant on registered APNAccounts from Claimer pallet has stalled out here -> https://github.com/Greenetwork/BLX_chain/tree/assets_integration
		// code below has issues with borrowing and ownership :/
		// #[weight = 0]
		// pub fn issue_token_airdrop(origin, apn: Vec<<T::Lookie as StaticLookup>::Source>, atokens: T::Balance1) -> DispatchResult {
		// 	let _sender = ensure_signed(origin)?;

		// 	let asset_id = Self::next_asset_id();

		// 	let length = Self::calculate_length(&apn);

		// 	for i in 0..length {
		// 		let current_apn = apn[i];
		// 		let apn_acc = T::Lookie::lookup(current_apn)?;
		// 		<Balances<T>>::insert(asset_id, &apn_acc, &atokens);
		// 	}

		// 	Ok(())
		// }

		// Automatic airdrop dependant on registered APNAccounts from Claimer pallet has stalled out here -> https://github.com/Greenetwork/BLX_chain/tree/assets_integration
		// this version requires AccountIDs
		#[weight = 0]
		pub fn issue_token_airdrop(origin, apn: Vec<T::AccountId>, atokens: T::Balance1) -> DispatchResult {
			let _sender = ensure_signed(origin)?;

			let asset_id = Self::next_asset_id();

			for i in 0..apn.len() {
				<Balances<T>>::insert(asset_id, &apn[i], &atokens);
			}
			Ok(())
		}
	}
}

impl<T:Config> Module<T> {

	pub fn calculate_length(
		vec_apn: &Vec<<T::Lookie as StaticLookup>::Source>
	) -> usize {
		vec_apn.len()
	}

}
