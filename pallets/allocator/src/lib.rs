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
	},
	Parameter,
	weights::{Weight, GetDispatchInfo},
};
use frame_system::{
	self as system, ensure_signed, ensure_none};

use sp_runtime::{
	RuntimeDebug,
	traits::{
		AtLeast32BitUnsigned, Zero, StaticLookup, Saturating, CheckedSub, CheckedAdd, Member,
	}
};

use codec::{Encode, Decode, HasCompact};

pub use pallet_assets::WeightInfo;

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
// pub type FrozenBalance<T> = <T as pallet_assets::Config>::FrozenBalance;

// pub type AssetIdOf<T> = <T as pallet_assets::Config>::AssetId;
// pub type AssetBalanceOf<T> = <T as pallet_assets::Config>::Balance;


pub trait Config: frame_system::Config { 
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The units in which we record balances.
		type Balance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy;

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
}

decl_storage! {
	trait Store for Module<T: Config> as Asset {
		/// Next available ID for user-created asset.
		pub NextAssetId get(fn next_asset_id) : T::AssetId;

		pub Balances:
			double_map hasher(twox_64_concat) T::AssetId, hasher(blake2_128_concat) T::AccountId => T::Balance;

		pub TotalSupply:
			map hasher(blake2_128_concat) T::AssetId => T::Balance;

		QueueAvailable get(fn queue_available): bool;
	}
}

decl_event!(
	pub enum Event<T> where
		<T as frame_system::Config>::AccountId,
		<T as Config>::Balance,
		<T as Config>::AssetId,
		// AssetOptions = AssetOptions<<T as Config>::Balance, <T as frame_system::Config>::AccountId>
	{
		/// Asset created. [asset_id, creator, asset_options]
		// Created(AssetId, AccountId, AssetOptions),
		/// Asset transfer succeeded. [asset_id, from, to, amount]
		Transferred(AssetId, AccountId, AccountId, Balance),
		/// Asset permission updated. [asset_id, new_permissions]
		// PermissionUpdated(AssetId, PermissionLatest<AccountId>),
		/// New asset minted. [asset_id, account, amount]
		Minted(AssetId, AccountId, Balance),
		/// Asset burned. [asset_id, account, amount]
		Burned(AssetId, AccountId, Balance),
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

		#[weight = 0]
		pub fn issue_token_airdrop(origin, atokens: T::Balance) -> DispatchResult {
			let _ = ensure_signed(origin)?;

			// const ACCOUNT_ALICE: u64 = 1;
			// const ACCOUNT_BOB: u64 = 2;
			// const COUNT_AIRDROP_RECIPIENTS: u64 = 2;
			

			// ensure!(!COUNT_AIRDROP_RECIPIENTS.is_zero(), ArithmeticError::DivisionByZero);

			let asset_id = Self::next_asset_id();

			// <NextAssetId<T>>::mutate(|asset_id| *asset_id += 1);
			// <Balances<T>>::insert(asset_id, &ACCOUNT_ALICE, TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
			// <Balances<T>>::insert(asset_id, &ACCOUNT_BOB, TOKENS_FIXED_SUPPLY / COUNT_AIRDROP_RECIPIENTS);
			// <TotalSupply<T>>::insert(asset_id, &atokens);

			// Self::deposit_event(RawEvent::Issued(asset_id, sender, TOKENS_FIXED_SUPPLY));
			Ok(())
		}
	}
}