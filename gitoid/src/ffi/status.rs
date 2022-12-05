/// Status codes for functions returning `c_int` to signal errors.
pub(crate) enum Status {
    /// Unknown error that shouldn't happen.
    UnexpectedError = -1,
    /// The buffer passed in is null.
    BufferIsNull = -2,
    /// The buffer passed in is too small to put data into.
    BufferTooSmall = -3,
    /// Writing to the provided buffer failed.
    BufferWriteFailed = -4,
}
