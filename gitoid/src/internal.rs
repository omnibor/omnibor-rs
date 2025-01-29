//! A gitoid representing a single artifact.

use crate::{Error, GitOid, HashAlgorithm, ObjectType, Result};
use core::marker::PhantomData;
use digest::{block_buffer::generic_array::GenericArray, Digest, OutputSizeUser};

/// Generate a GitOid from data in a buffer of bytes.
///
/// If data is small enough to fit in memory, then generating a GitOid for it
/// this way should be much faster, as it doesn't require seeking.
pub(crate) fn gitoid_from_buffer<H, O>(
    digester: H::Alg,
    reader: &[u8],
    expected_len: usize,
) -> Result<GitOid<H, O>>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    let expected_hash_length = <H::Alg as OutputSizeUser>::output_size();
    let (hash, amount_read) = hash_from_buffer::<H::Alg, O>(digester, reader, expected_len)?;

    if amount_read != expected_len {
        return Err(Error::UnexpectedReadLength {
            expected: expected_len,
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

#[cfg(feature = "std")]
pub(crate) use standard_impls::gitoid_from_reader;

#[cfg(feature = "async")]
pub(crate) use async_impls::gitoid_from_async_reader;

/// Helper function which actually applies the [`GitOid`] construction rules.
///
/// This function handles actually constructing the hash with the GitOID prefix,
/// and delegates to a buffered reader for performance of the chunked reading.
fn hash_from_buffer<D, O>(
    mut digester: D,
    buffer: &[u8],
    expected_len: usize,
) -> Result<(GenericArray<u8, D::OutputSize>, usize)>
where
    D: Digest,
    O: ObjectType,
{
    let hashed_len = expected_len - num_carriage_returns_in_buffer(buffer);
    digest_gitoid_header(&mut digester, O::NAME, hashed_len);
    digest_with_normalized_newlines(&mut digester, buffer);
    Ok((digester.finalize(), expected_len))
}

/// Digest the "header" required for a GitOID.
#[inline]
fn digest_gitoid_header<D>(digester: &mut D, object_type: &str, object_len: usize)
where
    D: Digest,
{
    digester.update(object_type.as_bytes());
    digester.update(b" ");
    digester.update(object_len.to_string().as_bytes());
    digester.update(b"\0");
}

/// Update a hash digest with data in a buffer, normalizing newlines.
#[inline]
fn digest_with_normalized_newlines<D>(digester: &mut D, buf: &[u8])
where
    D: Digest,
{
    for chunk in buf.chunk_by(|char1, _| *char1 != b'\r') {
        let chunk = match chunk.last() {
            // Omit the carriage return at the end of the chunk.
            Some(b'\r') => &chunk[0..(chunk.len() - 1)],
            _ => chunk,
        };

        digester.update(chunk)
    }
}

/// Count carriage returns in an in-memory buffer.
#[inline(always)]
fn num_carriage_returns_in_buffer(buffer: &[u8]) -> usize {
    bytecount::count(buffer, b'\r')
}

#[cfg(feature = "std")]
mod standard_impls {
    use crate::{
        util::for_each_buf_fill::ForEachBufFill as _, Error, GitOid, HashAlgorithm, ObjectType,
        Result,
    };
    use core::marker::PhantomData;
    use digest::{block_buffer::generic_array::GenericArray, Digest, OutputSizeUser};
    use std::io::{BufReader, Read, Seek, SeekFrom};

    /// Generate a GitOid by reading from an arbitrary reader.
    pub(crate) fn gitoid_from_reader<H, O, R>(
        digester: H::Alg,
        reader: R,
        expected_len: usize,
    ) -> Result<GitOid<H, O>>
    where
        H: HashAlgorithm,
        O: ObjectType,
        R: Read + Seek,
    {
        let expected_hash_length = <H::Alg as OutputSizeUser>::output_size();
        let (hash, amount_read) = hash_from_reader::<H::Alg, O, R>(digester, reader, expected_len)?;

        if amount_read != expected_len {
            return Err(Error::UnexpectedReadLength {
                expected: expected_len,
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

    /// Read a seek-able stream and reset to the beginning when done.
    fn read_and_reset<R, F>(reader: R, f: F) -> Result<(usize, R)>
    where
        R: Read + Seek,
        F: Fn(R) -> Result<(usize, R)>,
    {
        let (data, mut reader) = f(reader)?;
        reader.seek(SeekFrom::Start(0))?;
        Ok((data, reader))
    }

    /// Count carriage returns in a reader.
    fn num_carriage_returns_in_reader<R>(reader: R) -> Result<(usize, R)>
    where
        R: Read + Seek,
    {
        read_and_reset(reader, |reader| {
            let mut buf_reader = BufReader::new(reader);
            let mut total_dos_newlines = 0;

            buf_reader.for_each_buf_fill(|buf| {
                // The number of separators is the number of chunks minus one.
                total_dos_newlines += buf.chunk_by(|char1, _| *char1 != b'\r').count() - 1
            })?;

            Ok((total_dos_newlines, buf_reader.into_inner()))
        })
    }

    /// Helper function which actually applies the [`GitOid`] construction rules.
    ///
    /// This function handles actually constructing the hash with the GitOID prefix,
    /// and delegates to a buffered reader for performance of the chunked reading.
    fn hash_from_reader<D, O, R>(
        mut digester: D,
        reader: R,
        expected_len: usize,
    ) -> Result<(GenericArray<u8, D::OutputSize>, usize)>
    where
        D: Digest,
        O: ObjectType,
        R: Read + Seek,
    {
        let (num_carriage_returns, reader) = num_carriage_returns_in_reader(reader)?;
        let hashed_len = expected_len - num_carriage_returns;

        super::digest_gitoid_header(&mut digester, O::NAME, hashed_len);
        let amount_read = BufReader::new(reader)
            .for_each_buf_fill(|b| super::digest_with_normalized_newlines(&mut digester, b))?;

        Ok((digester.finalize(), amount_read))
    }
}

#[cfg(feature = "async")]
mod async_impls {
    use crate::{Error, GitOid, HashAlgorithm, ObjectType, Result};
    use core::marker::PhantomData;
    use digest::{block_buffer::generic_array::GenericArray, Digest, OutputSizeUser};
    use std::io::SeekFrom;
    use tokio::io::{
        AsyncBufReadExt as _, AsyncRead, AsyncSeek, AsyncSeekExt as _, BufReader as AsyncBufReader,
    };

    use super::digest_with_normalized_newlines;

    /// Async version of `gitoid_from_reader`.
    pub(crate) async fn gitoid_from_async_reader<H, O, R>(
        digester: H::Alg,
        reader: R,
        expected_len: usize,
    ) -> Result<GitOid<H, O>>
    where
        H: HashAlgorithm,
        O: ObjectType,
        R: AsyncRead + AsyncSeek + Unpin,
    {
        let expected_hash_len = <H::Alg as OutputSizeUser>::output_size();
        let (hash, amount_read) =
            hash_from_async_buffer::<H::Alg, O, R>(digester, reader, expected_len).await?;

        if amount_read != expected_len {
            return Err(Error::UnexpectedHashLength {
                expected: expected_len,
                observed: amount_read,
            });
        }

        if hash.len() != expected_hash_len {
            return Err(Error::UnexpectedHashLength {
                expected: expected_hash_len,
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
        expected_len: usize,
    ) -> Result<(GenericArray<u8, D::OutputSize>, usize)>
    where
        D: Digest,
        O: ObjectType,
        R: AsyncRead + AsyncSeek + Unpin,
    {
        let (num_carriage_returns, reader) = num_carriage_returns_in_async_reader(reader).await?;
        let hashed_len = expected_len - num_carriage_returns;

        super::digest_gitoid_header(&mut digester, O::NAME, hashed_len);

        let mut reader = AsyncBufReader::new(reader);
        let mut total_read = 0;

        loop {
            let buffer = reader.fill_buf().await?;
            let amount_read = buffer.len();

            if amount_read == 0 {
                break;
            }

            digest_with_normalized_newlines(&mut digester, buffer);

            reader.consume(amount_read);
            total_read += amount_read;
        }

        Ok((digester.finalize(), total_read))
    }

    /// Count carriage returns in a reader.
    async fn num_carriage_returns_in_async_reader<R>(reader: R) -> Result<(usize, R)>
    where
        R: AsyncRead + AsyncSeek + Unpin,
    {
        let mut reader = AsyncBufReader::new(reader);
        let mut total_dos_newlines = 0;

        loop {
            let buffer = reader.fill_buf().await?;
            let amount_read = buffer.len();

            if amount_read == 0 {
                break;
            }

            total_dos_newlines += buffer.chunk_by(|char1, _| *char1 != b'\r').count() - 1;

            reader.consume(amount_read);
        }

        let (data, mut reader) = (total_dos_newlines, reader.into_inner());
        reader.seek(SeekFrom::Start(0)).await?;
        Ok((data, reader))
    }
}
