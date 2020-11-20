#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]
//! BLX claimer
//! This pallet a stakeholder claiming an ApnToken via parcel number
//! It is a WIP

use core::{convert::TryInto, fmt};
use frame_support::{
	//codec::{Decode, Encode}, // used for on-chain storage
	decl_event, decl_module, decl_storage, debug, decl_error, // used for all of the different macros
	dispatch::DispatchResult, // the returns from a dispatachable call which is a function that a user can call as part of an extrensic
	ensure, // used to verify things
	storage::{StorageDoubleMap, StorageMap, StorageValue}, // storage types used
	traits::{Get, Imbalance}
};

// use water_balance::WaterCurrency;
// use account_set::AccountSet;
// use sp_std::collections::btree_set::BTreeSet;


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

/// Defines application identifier for crypto keys of this module.
///
/// Every module that deals with signatures needs to declare its unique identifier for
/// its crypto keys.
/// When offchain worker is signing transactions it's going to request keys of type
/// `KeyTypeId` from the keystore and use the ones it finds to sign the transaction.
/// The keys can be inserted manually via RPC (see `author_insertKey`).
pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");
pub const NUM_VEC_LEN: usize = 10;

// We are fetching information from github public API about organisation `substrate-developer-hub`.
pub const HTTP_REMOTE_REQUEST_BYTES: &[u8] = b"https://spencerbh.github.io/sandbox/18102019manualstrip.json";
pub const HTTP_HEADER_USER_AGENT: &[u8] = b"spencerbh";

/// Based on the above `KeyTypeId` we need to generate a pallet-specific crypto type wrappers.
/// We can use from supported crypto kinds (`sr25519`, `ed25519` and `ecdsa`) and augment
/// the types with this pallet-specific identifier.
pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;
	// implemented for ocw-runtime
	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	// implemented for mock runtime in test
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

/// This is the pallet's configuration trait
pub trait Trait: balances::Trait + system::Trait + CreateSignedTransaction<Call<Self>> {
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
	/// The overarching dispatch call type.
	type Call: From<Call<Self>>;
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	//type WaterBalanceSource: WaterBalance<Balance = Self::Balance>;
	// type Currency: WaterCurrency;//<Self::Balancey>;
}

// type BalanceyOf<T> = <<T as Trait>::Currency as WaterCurrency>::Balancey;

// Custom data type
#[derive(Debug)]
enum TransactionType {
	SignedSubmitNumber,
	UnsignedSubmitNumber,
	//HttpFetching,
	None,
}

/////////////////////////////////////////////////////////////////////////////////////////////////

pub type GroupIndex = u32; // this is Encode (which is necessary for double_map)

type Name = [u8; 32];

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct BasinOwnerId<AccountId> {
	pub owner: AccountId,
	pub basin_id: u32,
}

#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default, Debug)]
pub struct ApnToken<
	//Hash,
	AccountId,
	//Balance
	> {
	super_apn: u32,
	agency_name: Vec<u8>,
	area: u32,
	owner: AccountId,
	//balance: Balance,
	//annual_allocation: AnnualAllocation<Hash>, // needs to be converted to vector of structs or similar, review substrate kitties for more
}

//type ApnTokenOf<T> = ApnToken<
	//<T as system::Trait>::Hash, 
//	<T as balances::Trait>::Balance>;

//not sure we need this
// pub struct ApnAccount<T:Trait> {
// }

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct AnnualAllocation<Hash> {
	apn: u32,
	year: u32,
	hash: Hash, // this will be txn hash from Allocation Round
	current_years_allocation: u32,
}

//type AnnualAllocationOf<T> = AnnualAllocation<<T as system::Trait>::Hash,
 //<T as balances::Trait>::Balance
 //>;

 // TaskQueue, needs an extrinsic used to populate these fields
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default,Debug)]
pub struct TaskQueue {
	#[serde(deserialize_with = "de_string_to_bytes")]
	http_remote_reqst: Vec<u8>,
	#[serde(deserialize_with = "de_string_to_bytes")]
	http_header_usr: Vec<u8>,
}

 // TaskQueue, needs an extrinsic used to populate these fields
 #[serde(crate = "alt_serde")]
 #[derive(Deserialize, Encode, Decode, Default,Debug)]
 pub struct TaskQueueTwo {
 //	 basin: u32,
	 apn: u32,
 }

// Specifying serde path as `alt_serde`
// ref: https://serde.rs/container-attrs.html#crate
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct GithubInfo {
	// Specify our own deserializing function to convert JSON string to vector of bytes
	apn: u32,
	#[serde(deserialize_with = "de_string_to_bytes")]
	agencyname: Vec<u8>,
	shape_area: u32,
}

pub fn de_string_to_bytes<'de, D>(de: D) -> Result<Vec<u8>, D::Error>
where
	D: Deserializer<'de>,
{
	let s: &str = Deserialize::deserialize(de)?;
	Ok(s.as_bytes().to_vec())
}

impl fmt::Debug for GithubInfo {
	// `fmt` converts the vector of bytes inside the struct back to string for
	//   more friendly display.
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{{ apn: {}, agencyname: {}, shape_area: {} }}",
			&self.apn,
			str::from_utf8(&self.agencyname).map_err(|_| fmt::Error)?,
			&self.shape_area,
		)
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_storage! {
	trait Store for Module<T: Trait> as Dmap {
		
		/// Registration information for a given name
		pub Registration: map hasher(blake2_128_concat) Name => NameStatus<T::AccountId, T::BlockNumber, 
		//BalanceOf<T>
		>;

		/// The lookup from name to account
		pub Lookup: map hasher(blake2_128_concat) Name => Option<T::AccountId>; 

		// Get Apn Tokens from account_id, super_apn
		//pub ApnTokensBySuperApns get(fn super_things_by_super_apns):
		//	map hasher(blake2_128_concat) (T::AccountId, u32) => ApnToken;//<T::Balance>;

		// Get Apn Tokens from super_apn
		pub ApnTokensBySuperApns get(fn super_things_by_super_apns):
			map hasher(blake2_128_concat) u32 => ApnToken<
			T::AccountId,
			//T::Balance
			>;
		// Get ApnToken water balance from super_apn
		pub WaterBalanceBySuperApns get( fn water_balance_by_super_apns):
			map hasher(blake2_128_concat) u32 => u64;
		// Total Annual water budget in AF, will need to be spun out into seperate pallet for voting of what this number will be annually. 
		pub TotalAnnualBudget get(fn total_annual_budget): u64 = 2100;
		// not used at the moment
		//NextBasinId get (fn next_basin_id): u32;
		// Basin map
		//pub Basin get(fn basin): map hasher(blake2_128_concat) u32 => BasinOwnerId<T::AccountId>;
		// not WORKING, not sure which account's keys are being injected into keystore 
		// Balance of Apn_Tokens for owner, input is basin_id and AccountId
		pub BalanceApnTokens get(fn balance_apntokens): map hasher(blake2_128_concat) (u32, T::AccountId) => u32;
		// For membership, works
		//AllMembers get(fn all_members): Vec<T::AccountId>; 
		/// A map of TasksQueues to numbers
		TaskQueueByNumber get(fn task_queue_by_number):
			map hasher(blake2_128_concat) u32 => TaskQueueTwo;
		// A bool to track if there is a task in the queue to be fetched via HTTP
		QueueAvailable get(fn queue_available): bool;

		TaskNumber get(fn task_number): u32;

		// Testing type sharing between pallets for balances using trait, this is its pallet specific storage
		// pub TBalancey get (fn fetch_balance): 
		// 	map hasher(blake2_128_concat) u32 => BalanceyOf<T>;

		// The set of all members. Stored as a single vec
		//Members get(fn members): Vec<T::AccountId>;
	}
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_event! (
	pub enum Event<T>
	where 
		AccountId = <T as system::Trait>::AccountId,
		Balance = BalanceOf<T>,
		<T as frame_system::Trait>::BlockNumber,
	{
		//<T as system::Trait>::Hash,BalanceOfis accepted to contribute to the average.
		//NewNumber(Option<AccountId>, u64),
		/// New member for `AllMembers` group
		NewMember(AccountId),
		/// New ApnToken claimed event includes basin_id, super_apn
		NewApnTokenClaimed(u32,u32),
		// fields of the new allocation
		//NewAnnualAllocation(u32, Hash, Balance),
		// fields of the apn_token and the annual_allocation fields
		//NewApnTokenByExistingAnnualAllocation(u32, u32, Hash, Balance),
		// ""
		//NewApnTokenByNewAnnualAllocation(u32, u32, Hash, Balance),
		NameClaimed(Name, AccountId, BlockNumber),
		NameFreed(Name),
		NameSet(Name),
		NameAssigned(Name, AccountId),
		NameUnassigned(Name)
	}
);

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_error! {
	pub enum Error for Module<T: Trait> {
		// Error returned when making signed transactions in off-chain worker
		SignedSubmitNumberError,
		// Error returned when making remote http fetching
		HttpFetchingError0,
		HttpFetchingError1,
		HttpFetchingError2,
		HttpFetchingError3,
		HttpFetchingError4,
		HttpFetchingError5,
		HttpFetchingError6,
		HttpFetchingError7,
		HttpFetchingError8,
		HttpFetchingError9,
		// Error returned when gh-info has already been fetched
		AlreadyFetched,
		ApnsDontMatch,
		/// The current state of the name does not match this step in the state machine.
		UnexpectedState,
		/// The name provided does not follow the configured rules.
		InvalidName,
		/// The bid is invalid.
		InvalidBid,
		/// The claim is invalid.
		InvalidClaim,
		/// User is not the current bidder.
		NotBidder,
		/// The name has not expired in bidding or ownership.
		NotExpired,
		/// The name is already available.
		AlreadyAvailable,
		/// The name is permanent.
		Permanent,
		/// You are not the owner of this name.
		NotOwner,
		/// You are not assigned to this domain.
		NotAssigned,
		/// Ownership extensions are not available.
		NoExtensions,
	}
}

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Adds a new task to the TaskQueue
		#[weight = 0]
		pub fn insert_new_task(origin, 
			//task_number: u32, 
			//basin: u32, 
			apn: u32,
			amount: BalanceyOf<T>) -> DispatchResult {
			let _ = ensure_signed(origin)?;
			let task_queue = TaskQueueTwo {
				//basin,
				apn,
			};
			let task_number = 1;
			//let mut tbalancey = TBalanceyOf::<T>::get()
			// <TBalancey>::insert(&apn, amount);
			<TaskQueueByNumber>::insert(task_number, task_queue);
			QueueAvailable::put(true);
			Ok(())
		}

		/// Manually tell the chain there are no tasks in the task list, this is a hack, no bueno
		#[weight = 0]
		pub fn empty_tasks(origin) -> DispatchResult {
			QueueAvailable::put(false);
			Ok(())
		}

		fn set_name(origin, name: Name, state: NameStatus<T::AccountId, T::BlockNumber) {
			let who = ensure_signed(origin)?;
			Registration::<T>::insert(&name, state);
			Self::deposit_event(RawEvent::NameSet(name));
		}

		// Create the digital Basin with given no parameters, lol, useless for now
		//#[weight = 10_000]
		//fn create_basin(origin) -> DispatchResult {
		//	let member = ensure_signed(origin)?;
			// Generate next basin ID, 
		//	let basin_id = NextBasinId::get().checked_add(1).expect("basin id error");
		//	NextBasinId::put(basin_id);
			// Create new basin,
		//	let new_basin = BasinOwnerId {
		//		owner: member,
		//		basin_id: basin_id,
		//	};
			// Add new basin to map,
		//	<Basin<T>>::insert(basin_id, new_basin);
		//	Ok(())
		//}

		// Create an ApnToken with given parameters and link to existing basin
		//
		// @param basin_id used as basin identification
		// @param super_apn apn used as ID
		// @param agency_name 
		// @param area of APN related to ApnToken

		#[weight = 0]
		fn submit_apn_signed(origin, basin_id: u32, super_apn: u32, agency_name: Vec<u8>, area: u32) -> DispatchResult {
			debug::info!("submit_apn_signed: {:?}", super_apn);
			let who = ensure_signed(origin)?;
			Self::update_apn(who, basin_id, super_apn, agency_name, area)
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain workers");

			let result = 
				if Self::queue_available() == true {
					debug::info!("there is a task in the queue");
					//QueueAvailable::put(false);
					debug::info!("the task status is {:?}", Self::queue_available());
					Self::fetch_if_needed()
				//DataAvailable::put(true);
				} else {
					debug::info!("executing signed extrinsic");
					Self::signed_submit_apn()
					//if let Err(e) = result { debug::error!("Error: {:?}", e); }
			};
		}
	}
}

// impl<T: Trait> WaterCurrency for Module<T> {
// 	type Balancey = BalanceyOf<T>;
//     type PositiveImbalance: Imbalance<Self::BalanceyOf<T>, Opposite = Self::NegativeImbalance>;
// 	type NegativeImbalance: Imbalance<Self::BalanceyOf<T>, Opposite = Self::PositiveImbalance>;
	
//     /// The balance of an apn 
//     fn findbalance (apn: u32) -> BalanceyOf<T> {
// 		Self::fetch_balance(&apn)
// 		//fetch_balance(&apn);
// 	}

// 	fn deposit_into_apn(apn: u32, value: Self::BalanceyOf<T>) -> Self::PositiveImbalance {
// 		<TBalancey>
// 	};

//     // The total amount of issuance in the system, aka total amount of allocated water in the system
//     // which has yet to be spent
// 	//fn total_unspent_waterbalance() -> T::Balance {
// 	//	Self::fetch_balance()
// 	//}
// }

impl<T: Trait> StaticLookup for Module<T> {
	type Source = MultiAddress<T::AccountId, ()>;
	type Target = T::AccountId;

	fn lookup(a: Self::Source) -> Result<Self::Target, LookupError> {
		match a {
			MultiAddress::Id(id) => Ok(id),
			MultiAddress::Hash256(hash) => {
				Lookup::<T>::get(hash).ok_or(LookupError)
			},
			_ => Err(LookupError),
		}
	}

	fn unlookup(a: Self::Target) -> Self::Source {
		MultiAddress::Id(a)
	}
}

impl<T: Trait> Module<T> {
	fn update_apn(who: T::AccountId, basin_id: u32, super_apn: u32, agency_name: Vec<u8>, area: u32) -> DispatchResult {
		debug::info!("some info from offchain woah --->  basin {:?} | apn {:?} | agency_name {:?} | area {:?}", basin_id, super_apn, agency_name, area);
		//let who = ensure_signed(who)?;
		let task_queue_thing = Self::task_queue_by_number(1);
		// fetching what APN we used to submit task
		let apn_bytes = task_queue_thing.apn; 
		// checking to make sure that the APN we used to submit the task with is in the json that we were returned via offchain worker
		ensure!(apn_bytes == super_apn, <Error<T>>::ApnsDontMatch);
		if apn_bytes == super_apn {
			debug::info!("matchy matchy")}
		// Create new ApnToken
		let apn_token = ApnToken::<
		T::AccountId, 
		//T::Balance
		> {
			super_apn,
			agency_name,
			area,
			owner: who,
			//balance: 1337 // this is balance of Acre-feet for the ApnToken, arbitrary number for now
		};

		// Inserts the ApnToken on-chain, mapping to the basin id and the super_apn
		//<ApnTokensBySuperApns<T>>::insert((basin_id, super_apn), apn_token); // this is for when we use the balance trait
		//<ApnTokensBySuperApns<T>>::insert((who, super_apn), apn_token); // for with future ownership
		<ApnTokensBySuperApns<T>>::insert(super_apn, apn_token);
		// Emits event
		Self::deposit_event(RawEvent::NewApnTokenClaimed(basin_id,super_apn));
		// Create the WaterBalance fill with 0 for now, will be added to in other pallets
		let emptytank = 200;
		<WaterBalanceBySuperApns>::insert(super_apn, emptytank);
		Ok(())
	}

		/// Check if we have fetched github info before. If yes, we use the cached version that is
	///   stored in off-chain worker storage `storage`. If no, we fetch the remote info and then
	///   write the info into the storage for future retrieval.
	fn fetch_if_needed() -> Result<(), Error<T>> {
		// Start off by creating a reference to Local Storage value.
		// Since the local storage is common for all offchain workers, it's a good practice
		// to prepend our entry with the pallet name.
		let s_info = StorageValueRef::persistent(b"offchain-demo::gh-info");
		let s_lock = StorageValueRef::persistent(b"offchain-demo::lock");

		// The local storage is persisted and shared between runs of the offchain workers,
		// and offchain workers may run concurrently. We can use the `mutate` function, to
		// write a storage entry in an atomic fashion.
		//
		// It has a similar API as `StorageValue` that offer `get`, `set`, `mutate`.
		// If we are using a get-check-set access pattern, we likely want to use `mutate` to access
		// the storage in one go.
		//
		// Ref: https://substrate.dev/rustdocs/v2.0.0-rc3/sp_runtime/offchain/storage/struct.StorageValueRef.html
		if let Some(Some(gh_info)) = s_info.get::<GithubInfo>() {
			// gh-info has already been fetched. Return early.
			debug::info!("cached gh-info 1: {:?}", gh_info);
			return Ok(());
		}

		// We are implementing a mutex lock here with `s_lock`
		let res: Result<Result<bool, bool>, Error<T>> = s_lock.mutate(|s: Option<Option<bool>>| {
			match s {
				// `s` can be one of the following:
				//   `None`: the lock has never been set. Treated as the lock is free
				//   `Some(None)`: unexpected case, treated it as AlreadyFetch
				//   `Some(Some(false))`: the lock is free
				//   `Some(Some(true))`: the lock is held

				// If the lock has never been set or is free (false), return true to execute `fetch_n_parse`
				None | Some(Some(false)) => Ok(true),

				// Otherwise, someone already hold the lock (true), we want to skip `fetch_n_parse`.
				// Covering cases: `Some(None)` and `Some(Some(true))`
				_ => Err(<Error<T>>::AlreadyFetched),
			}
		});

		// Cases of `res` returned result:
		//   `Err(<Error<T>>)` - lock is held, so we want to skip `fetch_n_parse` function.
		//   `Ok(Err(true))` - Another ocw is writing to the storage while we set it,
		//                     we also skip `fetch_n_parse` in this case.
		//   `Ok(Ok(true))` - successfully acquire the lock, so we run `fetch_n_parse`
		if let Ok(Ok(true)) = res {
			match Self::fetch_n_parse() {
				Ok(gh_info) => {
					// set gh-info into the storage and release the lock
					s_info.set(&gh_info);
					s_lock.set(&false);

					debug::info!("fetched gh-info: {:?}", gh_info);
				}
				Err(err) => {
					// release the lock
					s_lock.set(&false);
					return Err(err);
				}
			}
		}
		Ok(())
	}

	/// Fetch from remote and deserialize the JSON to a struct
	fn fetch_n_parse() -> Result<GithubInfo, Error<T>> {
		let resp_bytes = Self::fetch_from_remote().map_err(|e| {
			debug::error!("fetch_from_remote error: {:?}", e);
			<Error<T>>::HttpFetchingError0
		})?;

		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError1)?;
		// Print out our fetched JSON string
		debug::info!("{}", resp_str);

		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
		let gh_info: GithubInfo =
			serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError2)?;
		Ok(gh_info)
	}

	/// This function uses the `offchain::http` API to query the remote github information,
	///   and returns the JSON response as vector of bytes.
	fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
		// enter github access info - will be replaced with actual database
		let user_agent_bytes = HTTP_HEADER_USER_AGENT.to_vec();
		let remote_url_bytes = HTTP_REMOTE_REQUEST_BYTES.to_vec();
		//let user_agent = HTTP_HEADER_USER_AGENT.to_vec();
		// will be used later to ensure data being stored on chain matches the apn entered
		let task_queue_thing = Self::task_queue_by_number(1);
		let apn_bytes = task_queue_thing.apn;  

		let user_agent = str::from_utf8(&user_agent_bytes).map_err(|_| <Error<T>>::HttpFetchingError3)?;
		debug::info!("from the task queue --> {}", user_agent);

		let remote_url =
			str::from_utf8(&remote_url_bytes).map_err(|_| <Error<T>>::HttpFetchingError4)?;

		debug::info!("sending request to: {}", remote_url);

		// Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`. this is going to need to be fed the apn_bytes
		let request = rt_offchain::http::Request::get(remote_url);

		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
		let timeout = sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(3000));

		// For github API request, we also need to specify `user-agent` in http request header.
		//   See: https://developer.github.com/v3/#user-agent-required
		let pending = request
			.add_header(
				"User-Agent",
				str::from_utf8(&user_agent_bytes).map_err(|_| <Error<T>>::HttpFetchingError5)?,
			)
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|_| <Error<T>>::HttpFetchingError6)?;

		// By default, the http request is async from the runtime perspective. So we are asking the
		//   runtime to wait here.
		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
		//   ref: https://substrate.dev/rustdocs/v2.0.0-rc3/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError7)?
			.map_err(|_| <Error<T>>::HttpFetchingError8)?;

		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError9);
		}

		// Next we fully read the response body and collect it to a vector of bytes.
		Ok(response.body().collect::<Vec<u8>>())
	}

	fn signed_submit_apn() -> Result<(), Error<T>> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			debug::error!("No local account available -- boi"); // HELP HERE
			return Err(<Error<T>>::SignedSubmitNumberError);
		}
		let s_info = StorageValueRef::persistent(b"offchain-demo::gh-info");
		debug::info!("we got to here 0.1");

		if let Some(Some(gh_info)) = s_info.get::<GithubInfo>() {
			debug::info!("we got to here 0.2");
			debug::info!("cached gh-info in submit function: {:?}", gh_info.apn);
			let b_i = 2; // need to remember we have this hardcoded in here, assigning basin_id to arbitrary value of 2
			let s_a = gh_info.apn;
			let a_n = gh_info.agencyname;
			let a_a = gh_info.shape_area;

			let results = signer.send_signed_transaction(|_acct| {
				Call::submit_apn_signed(b_i, s_a, a_n.clone(), a_a)
			});
			for (acc, res) in &results {
				match res {
					Ok(()) => {
						debug::native::info!(
							"off-chain send_signed: acc: {:?}| apn: {:#?}",
							acc.id,
							s_a.clone()
						);
					}
					Err(e) => {
						//debug::error!("[{:?}] Failed in signed_submit_number: {:?}", acc.id, e);
						debug::error!("[{:?}] Failed in signed_submit_number", acc.id);
						return Err(<Error<T>>::SignedSubmitNumberError);
					}
				};
			}
		};

		Ok(())
	}
}