//! Hash algorithms supported for Artifact IDs.
//!
//! __See [Hash Algorithms and Hash Providers][idx] documentation for more info.__
//!
//! [idx]: crate#hash-algorithms-and-hash-providers

use {
    crate::util::sealed::Sealed,
    digest::{
        consts::U32,
        generic_array::{sequence::GenericSequence, GenericArray},
    },
    std::{fmt::Debug, ops::Deref},
};

#[cfg(doc)]
use crate::ArtifactId;

/// Marker trait for hash algorithms supported for constructing [`ArtifactId`]s.
///
/// This trait is sealed, meaning it can only be implemented within the
/// `omnibor` crate.
pub trait HashAlgorithm: Sealed {
    /// The name of the hash algorithm, to be written in the GitOid string.
    #[doc(hidden)]
    const NAME: &'static str;

    /// The array type generated by the hash.
    #[doc(hidden)]
    type Array: GenericSequence<u8>
        + FromIterator<u8>
        + Deref<Target = [u8]>
        + PartialEq
        + Eq
        + Clone
        + Copy
        + Debug;
}

/// Use SHA-256 as the hash algorithm.
pub struct Sha256 {
    #[doc(hidden)]
    _private: (),
}

impl Sealed for Sha256 {}

impl HashAlgorithm for Sha256 {
    const NAME: &'static str = "sha256";

    // A SHA-256 hash is 32 bytes long.
    type Array = GenericArray<u8, U32>;
}
