//! A gitoid representing a single artifact.

use crate::{gitoid::GITOID_URL_SCHEME, Error, GitOid, HashAlgorithm, ObjectType, Result};
use core::{marker::PhantomData, ops::Not as _};
use digest::{block_buffer::generic_array::GenericArray, OutputSizeUser};

#[cfg(feature = "std")]
use digest::block_buffer::generic_array::sequence::GenericSequence;

#[cfg(feature = "url")]
use {core::str::Split, url::Url};

#[cfg(feature = "url")]
pub(crate) struct GitOidUrlParser<'u, H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    url: &'u Url,

    segments: Split<'u, char>,

    #[doc(hidden)]
    _hash_algorithm: PhantomData<H>,

    #[doc(hidden)]
    _object_type: PhantomData<O>,
}

#[allow(dead_code)]
fn some_if_not_empty(s: &str) -> Option<&str> {
    s.is_empty().not().then_some(s)
}

#[cfg(feature = "url")]
impl<'u, H, O> GitOidUrlParser<'u, H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    pub(crate) fn new(url: &'u Url) -> GitOidUrlParser<'u, H, O> {
        GitOidUrlParser {
            url,
            segments: url.path().split(':'),
            _hash_algorithm: PhantomData,
            _object_type: PhantomData,
        }
    }

    pub(crate) fn parse(&mut self) -> Result<GitOid<H, O>> {
        self.validate_url_scheme()
            .and_then(|_| self.validate_object_type())
            .and_then(|_| self.validate_hash_algorithm())
            .and_then(|_| self.parse_hash())
            .map(|hash| GitOid {
                _phantom: PhantomData,
                value: H::array_from_generic(hash),
            })
    }

    fn validate_url_scheme(&self) -> Result<()> {
        if self.url.scheme() != GITOID_URL_SCHEME {
            return Err(Error::InvalidScheme(self.url.clone()));
        }

        Ok(())
    }

    fn validate_object_type(&mut self) -> Result<()> {
        let object_type = self
            .segments
            .next()
            .and_then(some_if_not_empty)
            .ok_or_else(|| Error::MissingObjectType(self.url.clone()))?;

        if object_type != O::NAME {
            return Err(Error::MismatchedObjectType { expected: O::NAME });
        }

        Ok(())
    }

    fn validate_hash_algorithm(&mut self) -> Result<()> {
        let hash_algorithm = self
            .segments
            .next()
            .and_then(some_if_not_empty)
            .ok_or_else(|| Error::MissingHashAlgorithm(self.url.clone()))?;

        if hash_algorithm != H::NAME {
            return Err(Error::MismatchedHashAlgorithm { expected: H::NAME });
        }

        Ok(())
    }

    fn parse_hash(&mut self) -> Result<GenericArray<u8, <H::Alg as OutputSizeUser>::OutputSize>> {
        let hex_str = self
            .segments
            .next()
            .and_then(some_if_not_empty)
            .ok_or_else(|| Error::MissingHash(self.url.clone()))?;

        // TODO(alilleybrinker): When `sha1` et al. move to generic-array 1.0,
        //                       update this to use the `arr!` macro.
        let mut value = GenericArray::generate(|_| 0);
        hex::decode_to_slice(hex_str, &mut value)?;

        let expected_size = <H::Alg as OutputSizeUser>::output_size();
        if value.len() != expected_size {
            return Err(Error::UnexpectedHashLength {
                expected: expected_size,
                observed: value.len(),
            });
        }

        Ok(value)
    }
}
