//! Utility functions for the FFI code.

use crate::ffi::error::Error;
use crate::ffi::status::Status;
use core::ffi::c_int;
use std::ffi::CString;
use std::io::Write as _;

/// Write a string slice to a C buffer.
///
/// This performs a write, including the null terminator and performing zeroization of any
/// excess in the destination buffer.
pub(crate) fn write_to_c_buf(src: &str, mut dst: &mut [u8]) -> c_int {
    // Ensure the string has the null terminator.
    let src = match CString::new(src.as_bytes()) {
        Ok(s) => s,
        Err(_) => return Status::UnexpectedError as c_int,
    };
    let src = src.as_bytes_with_nul();

    // Make sure the destination buffer is big enough.
    if dst.len() < src.len() {
        return Status::BufferTooSmall as c_int;
    }

    // Write the buffer.
    match dst.write_all(src) {
        Ok(()) => 0,
        Err(_) => Status::BufferWriteFailed as c_int,
    }
}

/// Check if a pointer is null, and if it is return the given error.
pub(crate) fn check_null<T>(ptr: *const T, error: Error) -> Result<(), Error> {
    if ptr.is_null() {
        return Err(error);
    }

    Ok(())
}
