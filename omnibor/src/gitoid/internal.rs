//! A gitoid representing a single artifact.

use crate::{
    error::ArtifactIdError,
    gitoid::GitOid,
    hash_algorithm::HashAlgorithm,
    object_type::ObjectType,
    util::{
        for_each_buf_fill::ForEachBufFill as _,
        stream_len::{async_stream_len, stream_len},
    },
};
use digest::{
    generic_array::{sequence::GenericSequence, GenericArray},
    DynDigest,
};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    marker::PhantomData,
};
use tokio::io::{
    AsyncBufReadExt as _, AsyncRead, AsyncSeek, AsyncSeekExt as _, BufReader as AsyncBufReader,
};

/// Generate a GitOid from data in a buffer of bytes.
///
/// If data is small enough to fit in memory, then generating a GitOid for it
/// this way should be much faster, as it doesn't require seeking.
pub(crate) fn gitoid_from_buffer<H, O>(
    digester: &mut dyn DynDigest,
    buffer: &[u8],
) -> Result<GitOid<H, O>, ArtifactIdError>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    let hashed_len = buffer.len() - num_carriage_returns_in_buffer(buffer);
    digest_gitoid_header(digester, O::NAME, hashed_len);
    digest_with_normalized_newlines(digester, buffer);

    let mut hash: GenericArray<u8, <H::Array as GenericSequence<u8>>::Length> =
        GenericArray::default();

    digester
        .finalize_into_reset(&mut hash[..])
        .map_err(ArtifactIdError::FailedToHashInput)?;

    Ok(GitOid {
        _phantom: PhantomData,
        value: <H as HashAlgorithm>::Array::from_iter(hash),
    })
}

/// Generate a GitOid by reading from an arbitrary reader.
pub(crate) fn gitoid_from_reader<H, O, R>(
    digester: &mut dyn DynDigest,
    mut reader: R,
) -> Result<GitOid<H, O>, ArtifactIdError>
where
    H: HashAlgorithm,
    O: ObjectType,
    R: Read + Seek,
{
    let expected_len = stream_len(&mut reader)? as usize;
    let (num_carriage_returns, reader) = num_carriage_returns_in_reader(reader)?;
    let hashed_len = expected_len - num_carriage_returns;

    digest_gitoid_header(digester, O::NAME, hashed_len);
    let _ = BufReader::new(reader)
        .for_each_buf_fill(|b| digest_with_normalized_newlines(digester, b))?;

    let mut hash: GenericArray<u8, <H::Array as GenericSequence<u8>>::Length> =
        GenericArray::default();

    digester
        .finalize_into_reset(&mut hash[..])
        .map_err(ArtifactIdError::FailedToHashInput)?;

    Ok(GitOid {
        _phantom: PhantomData,
        value: <H as HashAlgorithm>::Array::from_iter(hash),
    })
}

/// Async version of `gitoid_from_reader`.
pub(crate) async fn gitoid_from_async_reader<H, O, R>(
    digester: &mut (dyn DynDigest + Send),
    mut reader: R,
) -> Result<GitOid<H, O>, ArtifactIdError>
where
    H: HashAlgorithm,
    O: ObjectType,
    R: AsyncRead + AsyncSeek + Unpin,
{
    let expected_len = async_stream_len(&mut reader).await? as usize;

    let (num_carriage_returns, reader) = num_carriage_returns_in_async_reader(reader).await?;
    let hashed_len = expected_len - num_carriage_returns;

    digest_gitoid_header(&mut *digester, O::NAME, hashed_len);

    let mut reader = AsyncBufReader::new(reader);

    loop {
        let buffer = reader
            .fill_buf()
            .await
            .map_err(|source| ArtifactIdError::FailedRead(Box::new(source)))?;
        let amount_read = buffer.len();

        if amount_read == 0 {
            break;
        }

        digest_with_normalized_newlines(&mut *digester, buffer);

        reader.consume(amount_read);
    }

    let mut hash: GenericArray<u8, <H::Array as GenericSequence<u8>>::Length> =
        GenericArray::default();

    digester
        .finalize_into_reset(&mut hash[..])
        .map_err(ArtifactIdError::FailedToHashInput)?;

    Ok(GitOid {
        _phantom: PhantomData,
        value: <H as HashAlgorithm>::Array::from_iter(hash),
    })
}

/// Digest the "header" required for a GitOID.
#[inline]
fn digest_gitoid_header(digester: &mut dyn DynDigest, object_type: &str, object_len: usize) {
    digester.update(object_type.as_bytes());
    digester.update(b" ");
    digester.update(object_len.to_string().as_bytes());
    digester.update(b"\0");
}

/// Update a hash digest with data in a buffer, normalizing newlines.
#[inline]
fn digest_with_normalized_newlines(digester: &mut dyn DynDigest, buf: &[u8]) {
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

/// Read a seek-able stream and reset to the beginning when done.
fn read_and_reset<R, F>(reader: R, f: F) -> Result<(usize, R), ArtifactIdError>
where
    R: Read + Seek,
    F: Fn(R) -> Result<(usize, R), ArtifactIdError>,
{
    let (data, mut reader) = f(reader)?;
    reader
        .seek(SeekFrom::Start(0))
        .map_err(|source| ArtifactIdError::FailedSeek(SeekFrom::Start(0), Box::new(source)))?;
    Ok((data, reader))
}

/// Count carriage returns in a reader.
fn num_carriage_returns_in_reader<R>(reader: R) -> Result<(usize, R), ArtifactIdError>
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

/// Count carriage returns in a reader.
async fn num_carriage_returns_in_async_reader<R>(reader: R) -> Result<(usize, R), ArtifactIdError>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    let mut reader = AsyncBufReader::new(reader);
    let mut total_dos_newlines = 0;

    loop {
        let buffer = reader
            .fill_buf()
            .await
            .map_err(|source| ArtifactIdError::FailedRead(Box::new(source)))?;
        let amount_read = buffer.len();

        if amount_read == 0 {
            break;
        }

        total_dos_newlines += buffer.chunk_by(|char1, _| *char1 != b'\r').count() - 1;

        reader.consume(amount_read);
    }

    let (data, mut reader) = (total_dos_newlines, reader.into_inner());
    reader
        .seek(SeekFrom::Start(0))
        .await
        .map_err(|source| ArtifactIdError::FailedSeek(SeekFrom::Start(0), Box::new(source)))?;
    Ok((data, reader))
}
