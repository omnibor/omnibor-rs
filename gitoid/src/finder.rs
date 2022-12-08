use crate::Error;
use crate::GitOid;
use crate::HashAlgorithm;
use crate::ObjectType;
use crate::Result;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::path::Path;
use std::path::PathBuf;

/// Struct for finding artifacts based on matching GitOids.
///
/// It's constructed with a default hash algorithm and object type. This is so,
/// if the `Finder` needs to resolve something like a `&Path` or a seekable
/// reader to a `GitOid`, it knows what settings to apply to the `GitOid`
/// construction.
///
/// The `Finder` works by supplying an iterator of `GitOid`s on construction
/// against which the finding will be performed.
///
/// When finding, you supply an iterator of "providers", which can be:
///
/// 1. `GitOid`
/// 2. `&GitOid`
/// 3. `(I, GitOid)`
/// 4. `(I, &GitOid)`
/// 5. `&Path`
/// 6. `(&Path, HashAlgorithm)`
/// 7. `(&Path, ObjectType)`
/// 8. `(&Path, HashAlgorithm, ObjectType)`
/// 9. `(I, R) where R: Read + Seek`
/// 10. `(I, R, HashAlgorithm) where R: Read + Seek`
/// 11. `(I, R, ObjectType) where R: Read + Seek`
/// 12. `(I, R, HashAlgorithm, ObjectType) where R: Read + Seek`
///
/// With just `GitOid`s (options 1 and 2), the identifiers are the `GitOid`s
/// themselves.
///
/// When pairing `GitOid`s with an "identifier" type (options 3 and 4), the
/// identifiers indicate what artifact each `GitOid` represents (like a string
/// name or a path on the filesystem).
///
/// When providing a path, optionally with per-path overrides for
/// `HashAlgorithm` and/or `ObjectType` (options 6, 7, and 8), the paths are
/// the identifiers, and are also used to generate the `GitOid`s.
///
/// When pairing an identifier type with a reader and optional overrides for
/// `HashAlgorithm` and/or `ObjectType` (options 9, 10, 11, and 12), the
/// identifiers are used to identify each `GitOid`, and the readers are used
/// to generate the `GitOid`s.
pub struct Finder {
    default_hash_algorithm: HashAlgorithm,
    default_object_type: ObjectType,
    gitoids: HashSet<GitOid>,
}

impl Finder {
    /// Construct a new `Finder` for a set of `GitOid`s.
    pub fn for_gitoids<'a, I>(
        default_hash_algorithm: HashAlgorithm,
        default_object_type: ObjectType,
        gitoids: I,
    ) -> Self
    where
        I: IntoIterator<Item = &'a GitOid>,
    {
        Finder {
            default_hash_algorithm,
            default_object_type,
            gitoids: gitoids.into_iter().copied().collect(),
        }
    }

    /// Find a single match against a set of `GitOid`s.
    ///
    /// The `Result` indicates if an error arose in resolving the `GitOid`.
    /// The `Option` indicates if a matching `GitOid` was found.
    pub fn find<P>(&self, potential: P) -> (P::Id, Result<Option<GitOid>>)
    where
        P: IntoIdentifiedGitOid,
    {
        let (id, gitoid_result) =
            potential.into_identified_gitoid(self.default_hash_algorithm, self.default_object_type);
        (
            id,
            gitoid_result.map(|gitoid| self.gitoids.get(&gitoid).copied()),
        )
    }

    /// Find matches against a set of `GitOid`s.
    ///
    /// The `Result` indicates if an error arose in resolving the `GitOid`.
    /// The `Option` indicates if a matching `GitOid` was found.
    pub fn find_all<'s, I: 's, P>(
        &'s self,
        potentials: I,
    ) -> impl Iterator<Item = (P::Id, Result<Option<GitOid>>)> + 's
    where
        I: IntoIterator<Item = P>,
        P: IntoIdentifiedGitOid,
    {
        potentials.into_iter().map(|potential| self.find(potential))
    }
}

/// Things which can resolve into a `GitOid` with an identifier.
///
/// The identifier can be any type, with the intent being that it's something
/// which indicates the source object of the `gitoid`. In the context of
/// finding matching GitOids, having some identifier to relate the match back
/// to a concrete artifact is central requirement.
pub trait IntoIdentifiedGitOid {
    /// The identifier type associated with the implementor.
    type Id;

    /// Convert the implementor type into a `GitOid` with an identifier.
    ///
    /// `GitOid` construction may fail, hence the `(Self::Id, Result<GitOid>)`
    /// return type.
    ///
    /// `default_hash_algorithm` or `default_object_type` may be ignored by
    /// an implementor which "knows better" about the parameters which ought
    /// to be used when constructing the `GitOid` for the relevant artifact.
    fn into_identified_gitoid(
        self,
        default_hash_algorithm: HashAlgorithm,
        default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>);
}

impl IntoIdentifiedGitOid for GitOid {
    type Id = GitOid;

    fn into_identified_gitoid(
        self,
        _default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        (self, Ok(self))
    }
}

impl<'gitoid> IntoIdentifiedGitOid for &'gitoid GitOid {
    type Id = GitOid;

    fn into_identified_gitoid(
        self,
        _default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        (*self, Ok(*self))
    }
}

impl<I> IntoIdentifiedGitOid for (I, GitOid) {
    type Id = I;

    fn into_identified_gitoid(
        self,
        _default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        (self.0, Ok(self.1))
    }
}

impl<'gitoid, I> IntoIdentifiedGitOid for (I, &'gitoid GitOid) {
    type Id = I;

    fn into_identified_gitoid(
        self,
        _default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        (self.0, Ok(*self.1))
    }
}

impl<'a> IntoIdentifiedGitOid for &'a Path {
    type Id = PathBuf;

    fn into_identified_gitoid(
        self,
        default_hash_algorithm: HashAlgorithm,
        default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        let file = match File::open(self) {
            Ok(f) => f,
            Err(e) => return (self.to_owned(), Err(Error::from(e))),
        };

        IntoIdentifiedGitOid::into_identified_gitoid(
            (self.to_owned(), file),
            default_hash_algorithm,
            default_object_type,
        )
    }
}

impl<'a> IntoIdentifiedGitOid for (&'a Path, HashAlgorithm) {
    type Id = PathBuf;

    fn into_identified_gitoid(
        self,
        _default_hash_algorithm: HashAlgorithm,
        default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        IntoIdentifiedGitOid::into_identified_gitoid(self.0, self.1, default_object_type)
    }
}

impl<'a> IntoIdentifiedGitOid for (&'a Path, ObjectType) {
    type Id = PathBuf;

    fn into_identified_gitoid(
        self,
        default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        IntoIdentifiedGitOid::into_identified_gitoid(self.0, default_hash_algorithm, self.1)
    }
}

impl<'a> IntoIdentifiedGitOid for (&'a Path, HashAlgorithm, ObjectType) {
    type Id = PathBuf;

    fn into_identified_gitoid(
        self,
        _default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        IntoIdentifiedGitOid::into_identified_gitoid(self.0, self.1, self.2)
    }
}

impl<I, R> IntoIdentifiedGitOid for (I, R)
where
    R: Read + Seek,
{
    type Id = I;

    fn into_identified_gitoid(
        mut self,
        default_hash_algorithm: HashAlgorithm,
        default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        let reader = BufReader::new(&mut self.1);
        let gitoid =
            match GitOid::new_from_reader(default_hash_algorithm, default_object_type, reader) {
                Ok(g) => g,
                Err(e) => return (self.0, Err(e)),
            };

        IntoIdentifiedGitOid::into_identified_gitoid(
            (self.0, gitoid),
            default_hash_algorithm,
            default_object_type,
        )
    }
}

impl<I, R> IntoIdentifiedGitOid for (I, R, HashAlgorithm)
where
    R: Read + Seek,
{
    type Id = I;

    fn into_identified_gitoid(
        mut self,
        _default_hash_algorithm: HashAlgorithm,
        default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        IntoIdentifiedGitOid::into_identified_gitoid(
            (self.0, &mut self.1),
            self.2,
            default_object_type,
        )
    }
}

impl<I, R> IntoIdentifiedGitOid for (I, R, ObjectType)
where
    R: Read + Seek,
{
    type Id = I;

    fn into_identified_gitoid(
        mut self,
        default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        IntoIdentifiedGitOid::into_identified_gitoid(
            (self.0, &mut self.1),
            default_hash_algorithm,
            self.2,
        )
    }
}

impl<I, R> IntoIdentifiedGitOid for (I, R, HashAlgorithm, ObjectType)
where
    R: Read + Seek,
{
    type Id = I;

    fn into_identified_gitoid(
        mut self,
        _default_hash_algorithm: HashAlgorithm,
        _default_object_type: ObjectType,
    ) -> (Self::Id, Result<GitOid>) {
        IntoIdentifiedGitOid::into_identified_gitoid((self.0, &mut self.1), self.2, self.3)
    }
}
