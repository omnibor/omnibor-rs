use gitoid::GitOid;
use im::{HashSet, Vector};

/// A [persistent][wiki] collection of [git oids][git_scm].
///
/// Why persistent? While Rust and the borrow checker is great about ownership and
/// mutation, always knowing that a Ref will not change if passed as a parameter
/// to a function eliminates a class of errors.
///
/// [wiki]: https://en.wikipedia.org/wiki/Persistent_data_structure
/// [git_scm]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
#[derive(Clone, PartialOrd, Eq, Ord, Debug, Hash, PartialEq)]
pub struct OmniBor {
    git_oids: HashSet<GitOid>,
}

impl FromIterator<GitOid> for OmniBor {
    /// Create an OmniBor from many GitOids
    fn from_iter<T>(gitoids: T) -> Self
    where
        T: IntoIterator<Item = GitOid>,
    {
        let me = OmniBor::new();
        me.add_many(gitoids)
    }
}

impl OmniBor {
    /// Create a new instance
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            git_oids: HashSet::new(),
        }
    }

    /// Create a OmniBor from many GitOids
    pub fn new_from_iterator<I>(gitoids: I) -> Self
    where
        I: IntoIterator<Item = GitOid>,
    {
        let me = OmniBor::new();
        me.add_many(gitoids)
    }

    /// Add a `GitOid` hash to the `OmniBor`.
    ///
    /// Note that this creates a new persistent data structure under the hood.
    pub fn add(&self, gitoid: GitOid) -> Self {
        self.add_many([gitoid])
    }

    /// Append many `GitOid`s and return a new `OmniBor`
    pub fn add_many<I>(&self, gitoids: I) -> Self
    where
        I: IntoIterator<Item = GitOid>,
    {
        let mut updated = self.git_oids.clone(); // im::HashSet has O(1) cloning
        for gitoid in gitoids {
            updated = updated.update(gitoid);
        }
        Self { git_oids: updated }
    }

    /// Return the `Vector` of `GitOid`s.
    pub fn get_oids(&self) -> HashSet<GitOid> {
        self.git_oids.clone() // im::HashSet as O(1) cloning.
    }

    /// Get a sorted `Vector` of `GitOid`s.
    ///
    /// In some cases, getting a sorted `Vector` of oids is desirable.
    /// This function (cost O(n log n)) returns a `Vector` of sorted oids
    pub fn get_sorted_oids(&self) -> Vector<GitOid> {
        let mut ret = self.git_oids.clone().into_iter().collect::<Vector<_>>();
        ret.sort();
        ret
    }
}

#[cfg(test)]
mod tests {
    use gitoid::{GitOid, HashAlgorithm, ObjectType::Blob};
    use im::vector;

    use super::*;

    #[test]
    fn test_add() {
        let oid = GitOid::new_from_str(HashAlgorithm::Sha256, Blob, "Hello");
        assert_eq!(OmniBor::new().add(oid).get_sorted_oids(), vector![oid])
    }

    #[test]
    fn test_add_many() {
        let mut oids: Vector<GitOid> = vec!["eee", "Hello", "Cat", "Dog"]
            .into_iter()
            .map(|s| GitOid::new_from_str(HashAlgorithm::Sha256, Blob, s))
            .collect();

        let da_bom = OmniBor::new().add_many(oids.clone());
        oids.sort();
        assert_eq!(da_bom.get_sorted_oids(), oids);
    }

    #[test]
    fn test_add_gitoid_to_gitbom() {
        let input = "hello world".as_bytes();

        let generated_gitoid = GitOid::new_from_bytes(HashAlgorithm::Sha256, Blob, input);

        let new_gitbom = OmniBor::new();
        let new_gitbom = new_gitbom.add(generated_gitoid);

        assert_eq!(
            "fee53a18d32820613c0527aa79be5cb30173c823a9b448fa4817767cc84c6f03",
            new_gitbom.get_sorted_oids()[0].hash().as_hex()
        )
    }
}
