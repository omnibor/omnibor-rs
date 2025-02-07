use {
    crate::error::ArtifactIdError,
    std::io::{Seek, SeekFrom},
    tokio::io::{AsyncSeek, AsyncSeekExt as _},
};

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
pub(crate) fn stream_len<R>(mut stream: R) -> Result<u64, ArtifactIdError>
where
    R: Seek,
{
    let old_pos = stream
        .stream_position()
        .map_err(|source| ArtifactIdError::FailedCheckReaderPos(Box::new(source)))?;

    let len = stream
        .seek(SeekFrom::End(0))
        .map_err(|source| ArtifactIdError::FailedSeek(SeekFrom::End(0), Box::new(source)))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        stream.seek(SeekFrom::Start(old_pos)).map_err(|source| {
            ArtifactIdError::FailedSeek(SeekFrom::Start(old_pos), Box::new(source))
        })?;
    }

    Ok(len)
}

/// An async equivalent of `stream_len`.
pub(crate) async fn async_stream_len<R>(mut stream: R) -> Result<u64, ArtifactIdError>
where
    R: AsyncSeek + Unpin,
{
    let old_pos = stream
        .stream_position()
        .await
        .map_err(|source| ArtifactIdError::FailedCheckReaderPos(Box::new(source)))?;

    let len = stream
        .seek(SeekFrom::End(0))
        .await
        .map_err(|source| ArtifactIdError::FailedSeek(SeekFrom::End(0), Box::new(source)))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        stream
            .seek(SeekFrom::Start(old_pos))
            .await
            .map_err(|source| {
                ArtifactIdError::FailedSeek(SeekFrom::Start(old_pos), Box::new(source))
            })?;
    }

    Ok(len)
}
