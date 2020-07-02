#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::string_lit_as_bytes)]
//! BLX claimer
//! This pallet a stakeholder claiming an ApnToken via parcel number
//! It is a WIP

use frame_support::{
	codec::{Decode, Encode}, // used to on-chain storage
	decl_event, decl_module, decl_storage, debug, decl_error, // used for all of the different macros
	dispatch::DispatchResult, // the return from a dispatchable call which is a funciton that a use can call as part of an extrinsic
	ensure, // used to verify things
	storage::{StorageDoubleMap, StorageMap, StorageValue}, // storage types used
	traits::Get, // no idea
};
use frame_system::{
	self as system, ensure_signed, ensure_none, 
	offchain::{ // offchain worker imports
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
	transaction_validity::{ // offchain worker unsigned transaction checks...
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

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct BasinOwnerId<AccountId> {
	pub owner: AccountId,
	pub basin_id: u32,
}

#[derive(Encode, Decode, Default, RuntimeDebug)]
pub struct ApnToken<
	Balance> {
	super_apn: u32,
	area: u32,
	balance: Balance,
	agency: Vec<u8>,
}

type ApnTokenOf<T> = ApnToken<
	<T as balances::Trait>::Balance>;

/////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

decl_storage! {
	trait Store for Module<T: Trait> as Dmap {
		
		// Get Apn Tokens for AccountId, currently returning empty struct
		ApnTokensByAccount get(fn apn_tokens_by_account):
			map hasher(blake2_128_concat) T::AccountId => ApnTokenOf<T>;

		// Get Apn Tokens for Super Number, works
		// (u32,u32) is basin_id and super_apn
		ApnTokensBySuperApns get(fn super_things_by_super_apns):
			map hasher(blake2_128_concat) (u32,u32) => ApnToken<T::Balance>;

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
		<T as balances::Trait>::Balance,
		AccountId = <T as system::Trait>::AccountId,
	{
		/// Event generated when a new number is accepted to contribute to the average.
		NewNumber(Option<AccountId>, u64),
		/// New member for `AllMembers` group
		NewMember(AccountId),
		// fields of the new allocation
		NewApnTokenByNewAnnualAllocation(u32, u32, Balance),
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
		pub fn submit_info_signed(origin, apn_first: u64) -> DispatchResult {
			debug::info!("submit_info_signed: {:?}", apn_first);
			let who = ensure_signed(origin)?;
			Self::create_apntoken_fromocw(Some(who), apn_first)
		}

		fn offchain_worker(choice: u32) { // ingesting some integer which is going to choose tx type we want
			debug::info!("Entering off-chain workers"); // printing to log

			let result = match Self::choose_tx_type(choice) { // match the output of the choose_tx_type function to one of the items in the options below
				// the match results in another function being executed
				TransactionType::SignedSubmitNumber => Self::signed_submit_number(choice), // this match results in executing a function where things submitted on-chain
				TransactionType::HttpFetching => Self::fetch_if_needed(), // this match results in executing a function which fetches the info from online
				TransactionType::None => Ok(()) // this match results in nothing
			};

			if let Err(e) = result { debug::error!("Error: {:?}", e); } // if we have an error come from one of the executed functions pass it to this and log it
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

// Create an ApnToken with parameters from ocw and link to basin
		//
		// @param super_apn apn used as ID
		// @area area of APN related to ApnToken
		// @balance AcreFeet of water allocated to that ApnToken

		#[weight = 10_000]
		fn create_apntoken_fromocw(
			origin, apn_first
		) -> DispatchResult {
			let member = ensure_signed(origin)?;
		  // ensure message sender is blah
			//let new_balance_of_apntokens = 25//<BalanceApnTokens<T>>::get((basin_id, member.clone())) + 1; // create a variable which is the BalanceApnTokens + 1
		//<BalanceApnTokens<T>>::insert((basin_id, member.clone()), new_balance_of_apntokens); // update the amount of ApnTokens a user has by 1 using the line above
	//		
			// Create new ApnToken
			let apn_token = ApnToken {
				apn_first,
				area = 44,
				balance = 3, // this is balance of Acre-feet for the ApnToken
				agency = String::from("initial contents"),
			};

			<ApnTokensBySuperApns<T>>::insert((5, super_apn), apn_token); // create an ApnToken, linked to basin_id
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
			agency: Vec<u8>,
		) -> DispatchResult {
			let member = ensure_signed(origin)?; // ensure message sender is blah
			let new_balance_of_apntokens = <BalanceApnTokens<T>>::get((basin_id, member.clone())) + 1; // create a variable which is the BalanceApnTokens + 1
		<BalanceApnTokens<T>>::insert((basin_id, member.clone()), new_balance_of_apntokens); // update the amount of ApnTokens a user has by 1 using the line above
	//		
			// Create new ApnToken
			let apn_token = ApnToken {
				super_apn,
				area,
				balance, // this is balance of Acre-feet for the ApnToken
				agency,
			};

			<ApnTokensBySuperApns<T>>::insert((basin_id, super_apn), apn_token); // create an ApnToken, linked to basin_id
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

	pub fn choose_tx_type(choice: u32) -> TransactionType {
		// Decide what type of transaction to send based on block number.
		// Each block the offchain worker will send one type of transaction back to the chain.
		// First a signed transaction, then an unsigned transaction, then an http fetch and json parsing.
		match choice {
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
		let s_info = StorageValueRef::persistent(b"claimer::apn-info");
		let s_lock = StorageValueRef::persistent(b"claimer::lock");

		// The local storage is persisted and shared between runs of the offchain workers,
		// and offchain workers may run concurrently. We can use the `mutate` function, to
		// write a storage entry in an atomic fashion.
		//
		// It has a similar API as `StorageValue` that offer `get`, `set`, `mutate`.
		// If we are using a get-check-set access pattern, we likely want to use `mutate` to access
		// the storage in one go.
		//
		// Ref: https://substrate.dev/rustdocs/v2.0.0-alpha.7/sp_runtime/offchain/storage/struct.StorageValueRef.html
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
					// set apn-info into the storage and release the lock
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

	fn signed_submit_number() -> Result<(), Error<T>> {
		let signer = Signer::<T, T::AuthorityId>::all_accounts();
		if !signer.can_sign() {
			debug::error!("No local account available");
			return Err(<Error<T>>::SignedSubmitNumberError);
		}

		// Using `SubmitSignedTransaction` associated type we create and submit a transaction
		// representing the call, we've just created.
		// Submit signed will return a vector of results for all accounts that were found in the
		// local keystore with expected `KEY_TYPE`.
		//let submission: u64 = choice as u64;
		let apn_info_fetched = Self::fetch_n_parse().map_err(|_| "Failed to fetch info");
		let apn_first: u64 = apn_info_fetched.apn;


		let results = signer.send_signed_transaction(|_acct| {
			// We are just submitting the current block number back on-chain
			Call::submit_info_signed(apn_first)
		});

		for (acc, res) in &results {
			match res {
				Ok(()) => {
					debug::native::info!(
						"off-chain send_signed: acc: {:?}| number: {}",
						acc.id,
						apn_info_fetched,
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