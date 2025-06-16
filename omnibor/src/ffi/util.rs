//! Utility functions for the FFI code.

use {
    crate::ffi::{error::Error, status::Status},
    std::{
        ffi::{c_int, CString},
        io::Write as _,
    },
};

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

// License: adapted from https://github.com/fizyk20/generic-array, reused under MIT license.
/// A const reimplementation of the [`transmute`](core::mem::transmute) function,
/// avoiding problems when the compiler can't prove equal sizes.
///
/// # Safety
///
/// Treat this the same as [`transmute`](core::mem::transmute), or (preferably) don't use it at all.
#[inline(always)]
pub(crate) const unsafe fn const_transmute<A, B>(a: A) -> B {
    if std::mem::size_of::<A>() != std::mem::size_of::<B>() {
        panic!("Size mismatch for generic_array::const_transmute");
    }

    #[repr(C)]
    union Union<A, B> {
        a: std::mem::ManuallyDrop<A>,
        b: std::mem::ManuallyDrop<B>,
    }

    let a = std::mem::ManuallyDrop::new(a);
    std::mem::ManuallyDrop::into_inner(Union { a }.b)
}
