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
	self as system, ensure_signed, ensure_none,
	offchain::{ // off chain worker inputs
		AppCrypto, CreateSignedTransaction, SendSignedTransaction, Signer, SubmitTransaction,
	},
};
use sp_runtime::RuntimeDebug;
use sp_std::prelude::*; // imports a bunch of boiler plate

use sp_std::str; // string
use core::{convert::TryInto, fmt}; // converts the vector of bytes inside the struct back to string for more friendly display.
use sp_core::crypto::KeyTypeId; // for using keys for signed extrinsics

use sp_runtime::{
	offchain as rt_offchain, // offchain worker
	offchain::storage::StorageValueRef, // offchain worker storage
	transaction_validity::{ // offchain worker unsigned transaction checks....
		InvalidTransaction, TransactionPriority, TransactionSource, TransactionValidity,
		ValidTransaction,
	},
};


// We use `alt_serde`, and Xanewok-modified `serde_json` so that we can compile the program
//   with serde(features `std`) and alt_serde(features `no_std`).
use alt_serde::{Deserialize, Deserializer};


/////////////////////////////////////////////////////////////////////////////////////////////////////////////

//#[cfg(test)]
//mod tests;

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

//////////////////////////////////////////////////////////////////////////////////////////////////

// Specifying serde path as `alt_serde`
// ref: https://serde.rs/container-attrs.html#crate
#[serde(crate = "alt_serde")]
#[derive(Deserialize, Encode, Decode, Default)]
struct ApnTokenInfo {
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

// Method for the ApnTokenInfo (utility specified for this struct)
impl fmt::Debug for ApnTokenInfo {
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

/////////////////////////////////////////////////////////////////////////////////////////////////
 
 
/// This is the pallet's configuration trait
pub trait Trait: balances::Trait + system::Trait + CreateSignedTransaction<Call<Self>>{
	/// The identifier type for an offchain worker.
	type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
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

type ApnTokenOf<T> = ApnToken<
	//<T as system::Trait>::Hash, 
	<T as balances::Trait>::Balance>;


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
		Numbers get(fn numbers): Vec<u64>;

		// Annual Allocation (double map to Apn Tokens and AccountId), works
		AnnualAllocationsByApnTokenorAccount get(fn annual_allocations_by_apn_tokens_or_account):
			double_map hasher(blake2_128_concat) u32, hasher(blake2_128_concat) T::AccountId => AnnualAllocationOf<T>;
		
		// Get Apn Tokens for AccountId, currently returning empty struct
		ApnTokensByAccount get(fn apn_tokens_by_account):
			map hasher(blake2_128_concat) T::AccountId => ApnTokenOf<T>;

		// Get Apn Tokens for Super Number, works
		ApnTokensBySuperApns get(fn super_things_by_super_apns):
			map hasher(blake2_128_concat) (u32,u32) => ApnToken<
				//T::Hash, 
				T::Balance>;

		NextBasinId get (fn next_basin_id): u32;

		// Basin map
		pub Basin get(fn basin): map hasher(blake2_128_concat) u32 => BasinOwnerId<T::AccountId>;

		// Balance of Apn_Tokens for owner
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

		#[weight = 0]
		pub fn submit_number_signed(origin, number: u64) -> DispatchResult {
			debug::info!("submit_number_signed: {:?}", number);
			let who = ensure_signed(origin)?;
			Self::append_or_replace_number(Some(who), number)
		}

		fn offchain_worker(block_number: T::BlockNumber) {
			debug::info!("Entering off-chain workers");

			let result = match Self::choose_tx_type(block_number) {
				TransactionType::SignedSubmitNumber => Self::signed_submit_number(block_number),
				TransactionType::HttpFetching => Self::fetch_if_needed(),
				TransactionType::None => Ok(())
			};

			if let Err(e) = result { debug::error!("Error: {:?}", e); }
		}
	

		/// Join the `AllMembers` vec before joining a group
		#[weight = 10_000]
		fn join_all_members(origin) -> DispatchResult {
			let new_member = ensure_signed(origin)?;
			//ensure!(!Self::is_member(&new_member), "already a member, can't join");
			<AllMembers<T>>::append(&new_member);

			Self::deposit_event(RawEvent::NewMember(new_member));
			Ok(())
		}

//		/// Stores an `AnnualAllocation` struct in the storage map
//		#[weight = 10_000]
//		fn insert_annual_allocation(origin, number: u32, hash: T::Hash, balance: T::Balance) -> DispatchResult {
//			let _ = ensure_signed(origin)?;
//			let thing = AnnualAllocation {
//							number,
//							hash,
//							balance,
//						};
//			<AnnualAllocationsByNumbers<T>>::insert(number, thing);
//			Self::deposit_event(RawEvent::NewAnnualAllocation(number, hash, balance));
//			Ok(())
//		}

//		/// Stores a `SuperThing` struct in the storage map using an `InnerThing` that was already
//		/// stored
//		#[weight = 10_000]
//		fn insert_apn_token_with_existing_annual_allocation(origin, inner_number: u32, super_number: u32) -> DispatchResult {
//			let _ = ensure_signed(origin)?;
//			let annual_allocation = Self::annual_allocations_by_numbers(inner_number);
//			let apn_token = ApnToken {
//				super_number,
//				annual_allocation: annual_allocation.clone(),
//			};
//			<ApnTokensBySuperNumbers<T>>::insert(super_number, apn_token);
//			Self::deposit_event(RawEvent::NewApnTokenByExistingAnnualAllocation(
//				super_number, annual_allocation.number, annual_allocation.hash, annual_allocation.balance));
//			Ok(())
//		}

		
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

			let new_balance_of_apntokens = <BalanceApnTokens<T>>::get((basin_id, member.clone())) + 1;
			<BalanceApnTokens<T>>::insert((basin_id, member.clone()), new_balance_of_apntokens);
			
			// Create new ApnToken
			let apn_token = ApnToken {
				super_apn,
				area,
				balance, // this is balance of Acre-feet for the ApnToken
			};

			<ApnTokensBySuperApns<T>>::insert((basin_id, super_apn), apn_token);
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {
	/// Add a new number to the list.
	fn append_or_replace_number(who: Option<T::AccountId>, number: u64) -> DispatchResult {
		Numbers::mutate(|numbers| {
			// The append or replace logic. The `numbers` vector is at most `NUM_VEC_LEN` long.
			let num_len = numbers.len();

			if num_len < NUM_VEC_LEN {
				numbers.push(number);
			} else {
				numbers[num_len % NUM_VEC_LEN] = number;
			}

			// displaying the average
			let average = match num_len {
				0 => 0,
				_ => numbers.iter().sum::<u64>() / (num_len as u64),
			};

			debug::info!("Current average of numbers is: {}", average);
		});

		// Raise the NewNumber event
		Self::deposit_event(RawEvent::NewNumber(who, number));
		Ok(())
	}

	fn choose_tx_type(block_number: T::BlockNumber) -> TransactionType {
		// Decide what type of transaction to send based on block number.
		// Each block the offchain worker will send one type of transaction back to the chain.
		// First a signed transaction, then an unsigned transaction, then an http fetch and json parsing.
		match block_number.try_into().ok().unwrap() % 3 {
			0 => TransactionType::SignedSubmitNumber,
			2 => TransactionType::HttpFetching,
			_ => TransactionType::None,
		}
	}

	/// Check if we have fetched github info before. If yes, we use the cached version that is
	///   stored in off-chain worker storage `storage`. If no, we fetch the remote info and then
	///   write the info into the storage for future retrieval.
	fn fetch_if_needed() -> Result<(), Error<T>> {
		// Start off by creating a reference to Local Storage value.
		// Since the local storage is common for all offchain workers, it's a good practice
		// to prepend our entry with the pallet name.
		let s_info = StorageValueRef::persistent(b"blx-doublemap::apn-info");
		let s_lock = StorageValueRef::persistent(b"blx-doublemap::lock");

		// The local storage is persisted and shared between runs of the offchain workers,
		// and offchain workers may run concurrently. We can use the `mutate` function, to
		// write a storage entry in an atomic fashion.
		//
		// It has a similar API as `StorageValue` that offer `get`, `set`, `mutate`.
		// If we are using a get-check-set access pattern, we likely want to use `mutate` to access
		// the storage in one go.
		//
		// Ref: https://substrate.dev/rustdocs/v2.0.0-alpha.7/sp_runtime/offchain/storage/struct.StorageValueRef.html
//		if let Some(Some(gh_info)) = s_info.get::<GithubInfo>() {
			// gh-info has already been fetched. Return early.
//			debug::info!("cached gh-info: {:?}", gh_info);
//			return Ok(());
//		}

		if let Some(Some(apn_info)) = s_info.get::<ApnTokenInfo>() {
			// apn-info has already been fetched. Return early.
			debug::info!("cached apn-info: {:?}", apn_info);
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
				Ok(apn_info) => {
					// set gh-info into the storage and release the lock
					s_info.set(&apn_info);
					s_lock.set(&false);

					debug::info!("fetched apn-info: {:?}", apn_info);
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
	fn fetch_n_parse() -> Result<ApnTokenInfo, Error<T>> {
		let resp_bytes = Self::fetch_from_remote().map_err(|e| {
			debug::error!("fetch_from_remote error: {:?}", e);
			<Error<T>>::HttpFetchingError
		})?;

		let resp_str = str::from_utf8(&resp_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;
		// Print out our fetched JSON string
		debug::info!("{}", resp_str);

		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
		let apn_info: ApnTokenInfo =
			serde_json::from_str(&resp_str).map_err(|_| <Error<T>>::HttpFetchingError)?;
		Ok(apn_info)
	}

	/// This function uses the `offchain::http` API to query the remote github information,
	///   and returns the JSON response as vector of bytes.
	fn fetch_from_remote() -> Result<Vec<u8>, Error<T>> {
		let remote_url_bytes = HTTP_REMOTE_REQUEST_BYTES.to_vec();
		let user_agent = HTTP_HEADER_USER_AGENT.to_vec();
		let remote_url =
			str::from_utf8(&remote_url_bytes).map_err(|_| <Error<T>>::HttpFetchingError)?;

		debug::info!("sending request to: {}", remote_url);

		// Initiate an external HTTP GET request. This is using high-level wrappers from `sp_runtime`.
		let request = rt_offchain::http::Request::get(remote_url);

		// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
		let timeout = sp_io::offchain::timestamp().add(rt_offchain::Duration::from_millis(3000));

		// For github API request, we also need to specify `user-agent` in http request header.
		//   See: https://developer.github.com/v3/#user-agent-required
		let pending = request
			.add_header(
				"User-Agent",
				str::from_utf8(&user_agent).map_err(|_| <Error<T>>::HttpFetchingError)?,
			)
			.deadline(timeout) // Setting the timeout time
			.send() // Sending the request out by the host
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		// By default, the http request is async from the runtime perspective. So we are asking the
		//   runtime to wait here.
		// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
		//   ref: https://substrate.dev/rustdocs/v2.0.0-alpha.8/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
		let response = pending
			.try_wait(timeout)
			.map_err(|_| <Error<T>>::HttpFetchingError)?
			.map_err(|_| <Error<T>>::HttpFetchingError)?;

		if response.code != 200 {
			debug::error!("Unexpected http request status code: {}", response.code);
			return Err(<Error<T>>::HttpFetchingError);
		}

		// Next we fully read the response body and collect it to a vector of bytes.
		Ok(response.body().collect::<Vec<u8>>())
	}

	fn signed_submit_number(block_number: T::BlockNumber) -> Result<(), Error<T>> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			debug::error!("No local account available");
			return Err(<Error<T>>::SignedSubmitNumberError);
		}

		// Using `SubmitSignedTransaction` associated type we create and submit a transaction
		// representing the call, we've just created.
		// Submit signed will return a vector of results for all accounts that were found in the
		// local keystore with expected `KEY_TYPE`.
		let submission: u64 = block_number.try_into().ok().unwrap() as u64;
		let results = signer.send_signed_transaction(|_acct| {
			// We are just submitting the current block number back on-chain
			Call::submit_number_signed(submission)
		});

		for (acc, res) in &results {
			match res {
				Ok(()) => {
					debug::native::info!(
						"off-chain send_signed: acc: {:?}| number: {}",
						acc.id,
						submission
					);
				}
				Err(e) => {
					debug::error!("[{:?}] Failed in signed_submit_number: {:?}", acc.id, e);
					return Err(<Error<T>>::SignedSubmitNumberError);
				}
			};
		}
		Ok(())
	}
}