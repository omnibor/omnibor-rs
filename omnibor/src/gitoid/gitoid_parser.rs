//! A gitoid representing a single artifact.

use {
    crate::{
        error::ArtifactIdError,
        gitoid::{GitOid, GITOID_URL_SCHEME},
        hash_algorithm::HashAlgorithm,
        object_type::ObjectType,
        util::clone_as_boxstr::CloneAsBoxstr,
    },
    std::{marker::PhantomData, ops::Not as _, str::Split},
};

pub(crate) struct GitOidParser<'s, H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    source: &'s str,

    segments: Split<'s, char>,

    #[doc(hidden)]
    _hash_algorithm: PhantomData<H>,

    #[doc(hidden)]
    _object_type: PhantomData<O>,
}

fn some_if_not_empty(s: &str) -> Option<&str> {
    s.is_empty().not().then_some(s)
}

impl<'u, H, O> GitOidParser<'u, H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    pub(crate) fn new(s: &'u str) -> GitOidParser<'u, H, O> {
        GitOidParser {
            source: s,
            segments: s.split(':'),
            _hash_algorithm: PhantomData,
            _object_type: PhantomData,
        }
    }

    pub(crate) fn next_segment(&mut self) -> Option<&str> {
        self.segments.next().and_then(some_if_not_empty)
    }

    pub(crate) fn parse(&mut self) -> Result<GitOid<H, O>, ArtifactIdError> {
        self.validate_url_scheme()
            .and_then(|_| self.validate_object_type())
            .and_then(|_| self.validate_hash_algorithm())
            .and_then(|_| self.parse_hash())
            .map(|value| GitOid {
                _phantom: PhantomData,
                value,
            })
    }

    fn validate_url_scheme(&mut self) -> Result<(), ArtifactIdError> {
        let source = self.source.clone_as_boxstr();

        let scheme = self
            .next_segment()
            .ok_or(ArtifactIdError::MissingScheme(source))?;

        if scheme != GITOID_URL_SCHEME {
            return Err(ArtifactIdError::InvalidScheme(
                self.source.clone_as_boxstr(),
            ));
        }

        Ok(())
    }

    fn validate_object_type(&mut self) -> Result<(), ArtifactIdError> {
        let source = self.source.clone_as_boxstr();

        let object_type = self
            .next_segment()
            .ok_or(ArtifactIdError::MissingObjectType(source))?;

        if object_type != O::NAME {
            return Err(ArtifactIdError::MismatchedObjectType {
                expected: O::NAME.clone_as_boxstr(),
                got: object_type.clone_as_boxstr(),
            });
        }

        Ok(())
    }

    fn validate_hash_algorithm(&mut self) -> Result<(), ArtifactIdError> {
        let source = self.source.clone_as_boxstr();

        let hash_algorithm = self
            .next_segment()
            .ok_or(ArtifactIdError::MissingHashAlgorithm(source))?;

        if hash_algorithm != H::NAME {
            return Err(ArtifactIdError::MismatchedHashAlgorithm {
                expected: H::NAME.clone_as_boxstr(),
                got: hash_algorithm.clone_as_boxstr(),
            });
        }

        Ok(())
    }

    fn parse_hash(&mut self) -> Result<H::Array, ArtifactIdError> {
        let source = self.source.clone_as_boxstr();

        let hex_str = self
            .next_segment()
            .ok_or(ArtifactIdError::MissingHash(source))?;

        let decoded = hex::decode(hex_str).map_err(|source| {
            ArtifactIdError::InvalidHex(hex_str.clone_as_boxstr(), Box::new(source))
        })?;

        let value = <H as HashAlgorithm>::Array::from_iter(decoded);

        Ok(value)
    }
}
