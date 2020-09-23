#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{FullCodec, Codec, Encode, Decode, EncodeLike};
use sp_std::{prelude::*, result, marker::PhantomData, ops::Div, fmt::Debug};
use sp_core::u32_trait::Value as U32;
use sp_runtime::traits::{MaybeSerializeDeserialize, AtLeast32Bit, Saturating, TrailingZeroInput, Bounded, Zero,
    BadOrigin, AtLeast32BitUnsigned};

/// Types that implement the WaterBalance trait are able to supply a map of apns to values
/// semi-fungible asset 
pub trait WaterBalance {
    /// The balance of an ApnToken's water balance
    
    //type ApnBalance;
    type Balance: AtLeast32BitUnsigned + FullCodec + Copy + MaybeSerializeDeserialize + Debug +
    Default;

    /// The balance of an apn 
    fn findbalance (apn: u32) -> Self::Balance;
    //fn get_space(id: SpaceId) -> Result<SpaceForRoles<Self::AccountId>, DispatchError>;


    // The total amount of issuance in the system, aka total amount of allocated water in the system
    // which has yet to be spent
    //fn total_unspent_waterbalance() -> Self::Balance;
} 