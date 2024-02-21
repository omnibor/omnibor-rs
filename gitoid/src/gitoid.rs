//! A gitoid representing a single artifact.

use crate::Error;
use crate::HashAlgorithm;
use crate::ObjectType;
use crate::Result;
use core::cmp::Ordering;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Formatter;
use core::fmt::Result as FmtResult;
use core::hash::Hash;
use core::hash::Hasher;
use core::marker::PhantomData;
use core::ops::Not as _;
use core::str::FromStr;
use core::str::Split;
use digest::Digest;
use digest::OutputSizeUser;
use format_bytes::format_bytes;
use generic_array::sequence::GenericSequence;
use generic_array::GenericArray;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use tokio::io::AsyncBufReadExt as _;
use tokio::io::AsyncRead;
use tokio::io::AsyncSeek;
use tokio::io::AsyncSeekExt as _;
use tokio::io::BufReader as AsyncBufReader;
use url::Url;

/// A struct that computes [gitoids][g] based on the selected algorithm
///
/// [g]: https://git-scm.com/book/en/v2/Git-Internals-Git-Objects
#[repr(C)]
pub struct GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    #[doc(hidden)]
    _phantom: PhantomData<O>,

    #[doc(hidden)]
    value: H::Array,
}

const GITOID_URL_SCHEME: &str = "gitoid";

impl<H, O> GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    /// Create a new `GitOid` based on a slice of bytes.
    pub fn from_bytes<B: AsRef<[u8]>>(content: B) -> GitOid<H, O> {
        fn inner<H, O>(content: &[u8]) -> GitOid<H, O>
        where
            H: HashAlgorithm,
            O: ObjectType,
        {
            // PANIC SAFETY: We're reading from an in-memory buffer, so no IO errors can arise.
            gitoid_from_buffer(H::new(), content, content.len()).unwrap()
        }

        inner(content.as_ref())
    }

    /// Create a `GitOid` from a UTF-8 string slice.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str<S: AsRef<str>>(s: S) -> GitOid<H, O> {
        fn inner<H, O>(s: &str) -> GitOid<H, O>
        where
            H: HashAlgorithm,
            O: ObjectType,
        {
            GitOid::from_bytes(s.as_bytes())
        }

        inner(s.as_ref())
    }

    /// Create a `GitOid` from a reader.
    pub fn from_reader<R: Read + Seek>(mut reader: R) -> Result<GitOid<H, O>> {
        let expected_length = stream_len(&mut reader)? as usize;
        GitOid::from_reader_with_length(reader, expected_length)
    }

    /// Generate a `GitOid` from a reader, providing an expected length in bytes.
    pub fn from_reader_with_length<R: Read>(
        reader: R,
        expected_length: usize,
    ) -> Result<GitOid<H, O>> {
        gitoid_from_buffer(H::new(), reader, expected_length)
    }

    /// Generate a `GitOid` from an asynchronous reader.
    pub async fn from_async_reader<R: AsyncRead + AsyncSeek + Unpin>(
        mut reader: R,
    ) -> Result<GitOid<H, O>> {
        let expected_length = async_stream_len(&mut reader).await? as usize;
        GitOid::from_async_reader_with_length(reader, expected_length).await
    }

    /// Generate a `GitOid` from an asynchronous reader, providing an expected length in bytes.
    pub async fn from_async_reader_with_length<R: AsyncRead + Unpin>(
        reader: R,
        expected_length: usize,
    ) -> Result<GitOid<H, O>> {
        gitoid_from_async_buffer(H::new(), reader, expected_length).await
    }

    /// Construct a new `GitOid` from a `Url`.
    pub fn from_url(url: Url) -> Result<GitOid<H, O>> {
        GitOid::try_from(url)
    }

    /// Get a URL for the current `GitOid`.
    pub fn url(&self) -> Url {
        // PANIC SAFETY: We know that this is a valid URL;
        //               our `Display` impl is the URL representation.
        Url::parse(&self.to_string()).unwrap()
    }

    /// Get the underlying bytes of the hash.
    pub fn as_bytes(&self) -> &[u8] {
        &self.value[..]
    }

    /// Convert the hash to a hexadecimal string.
    pub fn as_hex(&self) -> String {
        hex::encode(self.as_bytes())
    }

    /// Get the hash algorithm used for the `GitOid`.
    pub const fn hash_algorithm(&self) -> &'static str {
        H::NAME
    }

    /// Get the object type of the `GitOid`.
    pub const fn object_type(&self) -> &'static str {
        O::NAME
    }

    /// Get the length of the hash in bytes.
    pub fn hash_len(&self) -> usize {
        <H::Alg as OutputSizeUser>::output_size()
    }
}

impl<H, O> FromStr for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<GitOid<H, O>> {
        Ok(GitOid::from_str(s))
    }
}

impl<H, O> Clone for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<H, O> Copy for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
}

impl<H, O> PartialEq<GitOid<H, O>> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn eq(&self, other: &GitOid<H, O>) -> bool {
        self.value == other.value
    }
}

impl<H, O> Eq for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
}

impl<H, O> PartialOrd<GitOid<H, O>> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<H, O> Ord for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<H, O> Hash for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn hash<H2>(&self, state: &mut H2)
    where
        H2: Hasher,
    {
        self.value.hash(state);
    }
}

impl<H, O> Debug for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("GitOid")
            .field("value", &self.value)
            .finish()
    }
}

impl<H, O> Display for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{}:{}:{}:{}",
            GITOID_URL_SCHEME,
            O::NAME,
            H::NAME,
            self.as_hex()
        )
    }
}

struct GitOidUrlParser<'u, H, O>
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

fn some_if_not_empty(s: &str) -> Option<&str> {
    s.is_empty().not().then_some(s)
}

impl<'u, H, O> GitOidUrlParser<'u, H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    fn new(url: &'u Url) -> GitOidUrlParser<'u, H, O> {
        GitOidUrlParser {
            url,
            segments: url.path().split(':'),
            _hash_algorithm: PhantomData,
            _object_type: PhantomData,
        }
    }

    fn parse(&mut self) -> Result<GitOid<H, O>> {
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
            return Err(Error::MismatchedObjectType {
                expected: O::NAME.to_string(),
                observed: object_type.to_string(),
            });
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
            return Err(Error::MismatchedHashAlgorithm {
                expected: H::NAME.to_string(),
                observed: hash_algorithm.to_string(),
            });
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

impl<H, O> TryFrom<Url> for GitOid<H, O>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    type Error = Error;

    fn try_from(url: Url) -> Result<GitOid<H, O>> {
        GitOidUrlParser::new(&url).parse()
    }
}

/// Take a `BufReader` and generate a hash based on the `GitOid`'s hashing algorithm.
///
/// Will return an `Err` if the `BufReader` generates an `Err` or if the
/// `expected_length` is different from the actual length.
///
/// Why the latter `Err`?
///
/// The prefix string includes the number of bytes being hashed and that's the
/// `expected_length`. If the actual bytes hashed differs, then something went
/// wrong and the hash is not valid.
fn gitoid_from_buffer<H, O, R>(
    digester: H::Alg,
    reader: R,
    expected_read_length: usize,
) -> Result<GitOid<H, O>>
where
    H: HashAlgorithm,
    O: ObjectType,
    R: Read,
{
    let expected_hash_length = <H::Alg as OutputSizeUser>::output_size();
    let (hash, amount_read) =
        hash_from_buffer::<H::Alg, O, R>(digester, reader, expected_read_length)?;

    if amount_read != expected_read_length {
        return Err(Error::UnexpectedHashLength {
            expected: expected_read_length,
            observed: amount_read,
        });
    }

    if hash.len() != expected_hash_length {
        return Err(Error::UnexpectedHashLength {
            expected: expected_hash_length,
            observed: hash.len(),
        });
    }

    Ok(GitOid {
        _phantom: PhantomData,
        value: H::array_from_generic(hash),
    })
}

// Helper extension trait to give a convenient way to iterate over
// chunks sized to the size of the internal buffer of the reader.
trait ForEachChunk: BufRead {
    // Takes a function to apply to each chunk, and returns if any
    // errors arose along with the number of bytes read in total.
    fn for_each_chunk(&mut self, f: impl FnMut(&[u8])) -> Result<usize>;
}

impl<R: BufRead> ForEachChunk for R {
    fn for_each_chunk(&mut self, mut f: impl FnMut(&[u8])) -> Result<usize> {
        let mut total_read = 0;

        loop {
            let buffer = self.fill_buf()?;
            let amount_read = buffer.len();

            if amount_read == 0 {
                break;
            }

            f(buffer);

            self.consume(amount_read);
            total_read += amount_read;
        }

        Ok(total_read)
    }
}

/// Helper function which actually applies the [`GitOid`] construction rules.
///
/// This function handles actually constructing the hash with the GitOID prefix,
/// and delegates to a buffered reader for performance of the chunked reading.
fn hash_from_buffer<D, O, R>(
    mut digester: D,
    reader: R,
    expected_read_length: usize,
) -> Result<(GenericArray<u8, D::OutputSize>, usize)>
where
    D: Digest,
    O: ObjectType,
    R: Read,
{
    digester.update(format_bytes!(
        b"{} {}\0",
        O::NAME.as_bytes(),
        expected_read_length
    ));
    let amount_read = BufReader::new(reader).for_each_chunk(|b| digester.update(b))?;
    let hash = digester.finalize();
    Ok((hash, amount_read))
}

/// Async version of `gitoid_from_buffer`.
async fn gitoid_from_async_buffer<H, O, R>(
    digester: H::Alg,
    reader: R,
    expected_read_length: usize,
) -> Result<GitOid<H, O>>
where
    H: HashAlgorithm,
    O: ObjectType,
    R: AsyncRead + Unpin,
{
    let expected_hash_length = <H::Alg as OutputSizeUser>::output_size();
    let (hash, amount_read) =
        hash_from_async_buffer::<H::Alg, O, R>(digester, reader, expected_read_length).await?;

    if amount_read != expected_read_length {
        return Err(Error::UnexpectedHashLength {
            expected: expected_read_length,
            observed: amount_read,
        });
    }

    if hash.len() != expected_hash_length {
        return Err(Error::UnexpectedHashLength {
            expected: expected_hash_length,
            observed: hash.len(),
        });
    }

    Ok(GitOid {
        _phantom: PhantomData,
        value: H::array_from_generic(hash),
    })
}

/// Async version of `hash_from_buffer`.
async fn hash_from_async_buffer<D, O, R>(
    mut digester: D,
    reader: R,
    expected_read_length: usize,
) -> Result<(GenericArray<u8, D::OutputSize>, usize)>
where
    D: Digest,
    O: ObjectType,
    R: AsyncRead + Unpin,
{
    digester.update(format_bytes!(
        b"{} {}\0",
        O::NAME.as_bytes(),
        expected_read_length
    ));

    let mut reader = AsyncBufReader::new(reader);

    let mut total_read = 0;

    loop {
        let buffer = reader.fill_buf().await?;
        let amount_read = buffer.len();

        if amount_read == 0 {
            break;
        }

        digester.update(buffer);

        reader.consume(amount_read);
        total_read += amount_read;
    }

    let hash = digester.finalize();
    Ok((hash, total_read))
}

// Adapted from the Rust standard library's unstable implementation
// of `Seek::stream_len`.
//
// TODO(abrinker): Remove this when `Seek::stream_len` is stabilized.
//
// License reproduction:
//
// Permission is hereby granted, free of charge, to any
// person obtaining a copy of this software and associated
// documentation files (the "Software"), to deal in the
// Software without restriction, including without
// limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software
// is furnished to do so, subject to the following
// conditions:
//
// The above copyright notice and this permission notice
// shall be included in all copies or substantial portions
// of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
// ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
// TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
// PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
// SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
// OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
// IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.
fn stream_len<R>(mut stream: R) -> Result<u64>
where
    R: Seek,
{
    let old_pos = stream.stream_position()?;
    let len = stream.seek(SeekFrom::End(0))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        stream.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len)
}

/// An async equivalent of `stream_len`.
async fn async_stream_len<R>(mut stream: R) -> Result<u64>
where
    R: AsyncSeek + Unpin,
{
    let old_pos = stream.stream_position().await?;
    let len = stream.seek(SeekFrom::End(0)).await?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        stream.seek(SeekFrom::Start(old_pos)).await?;
    }

    Ok(len)
}
