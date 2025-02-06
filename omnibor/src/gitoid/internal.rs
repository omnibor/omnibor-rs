//! A gitoid representing a single artifact.

use crate::{
    error::Error,
    gitoid::GitOid,
    hash_algorithm::HashAlgorithm,
    object_type::ObjectType,
    util::{
        for_each_buf_fill::ForEachBufFill as _,
        stream_len::{async_stream_len, stream_len},
    },
};
use digest::Digest;
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
    mut digester: impl Digest,
    buffer: &[u8],
) -> Result<GitOid<H, O>, Error>
where
    H: HashAlgorithm,
    O: ObjectType,
{
    let hashed_len = buffer.len() - num_carriage_returns_in_buffer(buffer);
    digest_gitoid_header(&mut digester, O::NAME, hashed_len);
    digest_with_normalized_newlines(&mut digester, buffer);
    let hash = digester.finalize();
    Ok(GitOid {
        _phantom: PhantomData,
        value: <H as HashAlgorithm>::Array::from_iter(hash),
    })
}

/// Generate a GitOid by reading from an arbitrary reader.
pub(crate) fn gitoid_from_reader<H, O, R>(
    mut digester: impl Digest,
    mut reader: R,
) -> Result<GitOid<H, O>, Error>
where
    H: HashAlgorithm,
    O: ObjectType,
    R: Read + Seek,
{
    let expected_len = stream_len(&mut reader)? as usize;
    let (num_carriage_returns, reader) = num_carriage_returns_in_reader(reader)?;
    let hashed_len = expected_len - num_carriage_returns;

    digest_gitoid_header(&mut digester, O::NAME, hashed_len);
    let _ = BufReader::new(reader)
        .for_each_buf_fill(|b| digest_with_normalized_newlines(&mut digester, b))?;

    let hash = digester.finalize();

    Ok(GitOid {
        _phantom: PhantomData,
        value: <H as HashAlgorithm>::Array::from_iter(hash),
    })
}

/// Async version of `gitoid_from_reader`.
pub(crate) async fn gitoid_from_async_reader<H, O, R>(
    mut digester: impl Digest,
    mut reader: R,
) -> Result<GitOid<H, O>, Error>
where
    H: HashAlgorithm,
    O: ObjectType,
    R: AsyncRead + AsyncSeek + Unpin,
{
    let expected_len = async_stream_len(&mut reader).await? as usize;

    let (num_carriage_returns, reader) = num_carriage_returns_in_async_reader(reader).await?;
    let hashed_len = expected_len - num_carriage_returns;

    digest_gitoid_header(&mut digester, O::NAME, hashed_len);

    let mut reader = AsyncBufReader::new(reader);

    loop {
        let buffer = reader.fill_buf().await?;
        let amount_read = buffer.len();

        if amount_read == 0 {
            break;
        }

        digest_with_normalized_newlines(&mut digester, buffer);

        reader.consume(amount_read);
    }

    let hash = digester.finalize();

    Ok(GitOid {
        _phantom: PhantomData,
        value: <H as HashAlgorithm>::Array::from_iter(hash),
    })
}

/// Digest the "header" required for a GitOID.
#[inline]
fn digest_gitoid_header(digester: &mut impl Digest, object_type: &str, object_len: usize) {
    digester.update(object_type.as_bytes());
    digester.update(b" ");
    digester.update(object_len.to_string().as_bytes());
    digester.update(b"\0");
}

/// Update a hash digest with data in a buffer, normalizing newlines.
#[inline]
fn digest_with_normalized_newlines(digester: &mut impl Digest, buf: &[u8]) {
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
fn read_and_reset<R, F>(reader: R, f: F) -> Result<(usize, R), Error>
where
    R: Read + Seek,
    F: Fn(R) -> Result<(usize, R), Error>,
{
    let (data, mut reader) = f(reader)?;
    reader.seek(SeekFrom::Start(0))?;
    Ok((data, reader))
}

/// Count carriage returns in a reader.
fn num_carriage_returns_in_reader<R>(reader: R) -> Result<(usize, R), Error>
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
async fn num_carriage_returns_in_async_reader<R>(reader: R) -> Result<(usize, R), Error>
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
