//! A source of data which will be used to produce a `GitOid`.

use pin_project::pin_project;
use std::io::Result as IOResult;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

/// Represents a source of data which will be read to produce a `GitOid`.
#[pin_project]
pub struct Source<R> {
    /// The reader itself.
    #[pin]
    reader: R,
    /// The length of the data being read.
    length: usize,
}

#[allow(clippy::len_without_is_empty)]
impl<R> Source<R> {
    /// Create a new `Source` based on a `reader` and `length`.
    pub fn new(reader: R, length: usize) -> Self {
        Self { reader, length }
    }

    /// Get the length of the read data.
    pub fn len(&self) -> usize {
        self.length
    }
}

impl<R: AsyncRead> AsyncRead for Source<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IOResult<()>> {
        self.project().reader.poll_read(cx, buf)
    }
}
