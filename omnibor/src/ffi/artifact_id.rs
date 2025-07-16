//! The main ArtifactId FFI functions.

use crate::hash_provider::RustCrypto;

use {
    crate::{
        artifact_id::ArtifactId,
        ffi::{
            error::{catch_panic, get_error_msg, Error},
            status::Status,
            util::{check_null, const_transmute, write_to_c_buf},
        },
        hash_algorithm::Sha256,
    },
    core::{
        ffi::{c_char, c_int, CStr},
        ptr::{null, null_mut},
        slice::{from_raw_parts, from_raw_parts_mut},
    },
    std::{ffi::CString, fs::File},
};

#[cfg(target_family = "unix")]
use std::os::unix::prelude::{FromRawFd, RawFd};

#[cfg(target_family = "windows")]
use std::os::windows::{io::FromRawHandle, prelude::RawHandle};
use std::str::FromStr;

/// Get the last-written error message written to a buffer.
///
/// If successful, it returns the number of bytes written to the buffer.
///
/// # Safety
///
/// The length passed must match the length of the buffer provided.
///
/// If the buffer pointer is null, the function will fail and return an
/// error code.
#[no_mangle]
pub unsafe extern "C" fn ob_error_message(buffer: *mut c_char, length: c_int) -> c_int {
    if buffer.is_null() {
        return Status::BufferIsNull as c_int;
    }

    let buffer = unsafe { from_raw_parts_mut(buffer as *mut u8, length as usize) };
    let last_err = get_error_msg().unwrap_or_default();
    write_to_c_buf(&last_err, buffer)
}

/// Embed a C string into the binary.
macro_rules! embed_cstr {
    ($name:ident, $arr:expr) => {
        /// cbindgen:ignore
        const $name: *const c_char = $arr.as_ptr();
    };
}

embed_cstr!(OBJECT_TYPE_BLOB, [0x62, 0x6C, 0x6F, 0x62, 0x00]);
embed_cstr!(
    HASH_ALGORITHM_SHA256,
    [0x73, 0x68, 0x61, 0x32, 0x35, 0x36, 0x00]
);

/// Free the given string.
///
/// Does nothing if the pointer is `NULL`.
///
/// # Safety
///
/// This function must only ever be called with strings obtained from another
/// `ArtifactId` FFI function and where the function documentation indicates that
/// the string needs to be freed.
#[no_mangle]
pub unsafe extern "C" fn ob_str_free(s: *const c_char) {
    if s.is_null() {
        return;
    }

    let _ = unsafe { CString::from_raw(s as *mut _) };
}

/// The number of bytes for a SHA-256 hash.
const NUM_SHA_256_BYTES: usize = 32;

/// A `ArtifactId` constructed with the specified hash algorithm.
pub type ArtifactIdSha256 = [u8; NUM_SHA_256_BYTES];

/// Construct a new `ArtifactId` from a buffer of bytes.
///
/// # Safety
///
/// `content_len` is the number of elements, not the number of bytes.
///
/// `content_len` times 8 (byte size) must be less than or equal to the
/// maximum size representable with an unsigned integer at the size used by
/// the ISA (32-bit or 64-bit usually).
///
/// `content` must not be null, and the length of the buffer must match the
/// length in bytes passed by `content_len`.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_id_bytes(
    content: *mut u8,
    content_len: usize,
) -> *const ArtifactIdSha256 {
    let output = catch_panic(|| {
        check_null(content, Error::ContentPtrIsNull)?;
        let content = unsafe { from_raw_parts(content, content_len) };
        let provider = RustCrypto::new();
        // SAFETY: Identifying bytes is infallible.
        let artifact_id = ArtifactId::new(provider, content).unwrap();
        let artifact_id: ArtifactIdSha256 = const_transmute(artifact_id);
        let boxed = Box::new(artifact_id);
        Ok(Box::into_raw(boxed) as *const _)
    });

    output.unwrap_or_else(null)
}

/// Construct a new `ArtifactId` from a C-string of data.
///
/// # Safety
///
/// The string passed _must_ be a valid C-string with a nul-terminator at the
/// end, all contained in a single contiguous allocation. The pointer must also
/// not be null.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_id_str(s: *const c_char) -> *const ArtifactIdSha256 {
    let output = catch_panic(|| {
        check_null(s, Error::StringPtrIsNull)?;
        let s = unsafe { CStr::from_ptr(s) }.to_str()?;
        let provider = RustCrypto::new();
        let artifact_id = ArtifactId::new(provider, s)?;
        let artifact_id: ArtifactIdSha256 = const_transmute(artifact_id);
        let boxed = Box::new(artifact_id);
        Ok(Box::into_raw(boxed) as *const _)
    });

    output.unwrap_or_else(null)
}

/// Construct a new `ArtifactId` from a `URL` in a C-string.
///
/// # Safety
///
/// If the pointer is null, an error is returned.
///
/// The returned `ArtifactId` must be freed.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_try_from_str(s: *const c_char) -> *const ArtifactIdSha256 {
    let output = catch_panic(|| {
        check_null(s, Error::StringPtrIsNull)?;
        let raw_str = unsafe { CStr::from_ptr(s) }.to_str()?;
        let s = raw_str.to_string();
        let artifact_id = ArtifactId::<Sha256>::from_str(&s)?;
        let artifact_id: ArtifactIdSha256 = const_transmute(artifact_id);
        let boxed = Box::new(artifact_id);
        Ok(Box::into_raw(boxed) as *const _)
    });

    output.unwrap_or_else(null)
}

/// Create a new `ArtifactId` by reading data from a file.
///
/// # Safety
///
/// The provided file descriptor must be valid and open for reading.
///
/// Returns an invalid `ArtifactId` if construction fails.
#[cfg(target_family = "unix")]
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_id_reader(fd: RawFd) -> *const ArtifactIdSha256 {
    let output = catch_panic(|| {
        let mut file = unsafe { File::from_raw_fd(fd) };
        let provider = RustCrypto::new();
        let artifact_id = ArtifactId::new(provider, &mut file)?;
        let artifact_id: ArtifactIdSha256 = const_transmute(artifact_id);
        let boxed = Box::new(artifact_id);
        Ok(Box::into_raw(boxed) as *const _)
    });

    output.unwrap_or_else(null)
}

/// Create a new `ArtifactId` by reading data from a file.
///
/// # Safety
///
/// The provided file handle must be valid and open for reading.
///
/// Returns an invalid `ArtifactId` if construction fails.
#[cfg(target_family = "windows")]
#[no_mangle]
/// cbindgen:ignore
pub unsafe extern "C" fn ob_aid_sha256_id_reader(handle: RawHandle) -> *const ArtifactIdSha256 {
    let output = catch_panic(|| {
        let mut file = unsafe { File::from_raw_handle(handle) };
        let provider = RustCrypto::new();
        let artifact_id = ArtifactId::new(provider, &mut file)?;
        let artifact_id: ArtifactIdSha256 = const_transmute(artifact_id);
        let boxed = Box::new(artifact_id);
        Ok(Box::into_raw(boxed) as *const _)
    });

    output.unwrap_or_else(null)
}

/// Construct a URL representation of a `ArtifactId`.
///
/// # Safety
///
/// The resulting string _must_ be freed with a call to `ob_str_free`.
///
/// Returns a `NULL` pointer if the URL construction fails.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_str(ptr: *const ArtifactIdSha256) -> *const c_char {
    let output = catch_panic(|| {
        check_null(ptr, Error::ArtifactIdPtrIsNull)?;
        let artifact_id = unsafe { &*ptr };
        let artifact_id: ArtifactId<Sha256> = const_transmute(artifact_id);
        let url = CString::new(artifact_id.to_string())?;
        Ok(url.into_raw() as *const _)
    });

    output.unwrap_or_else(null)
}

/// Get the name of an `ObjectType` as a C-string.
///
/// # Safety
///
/// Returns a null pointer if the string cannot be returned.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_object_type(ptr: *const ArtifactIdSha256) -> *const c_char {
    let output = catch_panic(|| {
        check_null(ptr, Error::ArtifactIdPtrIsNull)?;
        let artifact_id = unsafe { &*ptr };
        let artifact_id: ArtifactId<Sha256> = const_transmute(artifact_id);
        let object_type = artifact_id.object_type();

        match object_type {
            "blob" => Ok(OBJECT_TYPE_BLOB),
            _ => unimplemented!(),
        }
    });

    output.unwrap_or_else(null)
}

/// Get the length of the `ArtifactId` hash in bytes.
///
/// # Safety
///
/// If `ptr` is `null` then an error code of -5 is returned.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_hash_len(ptr: *const ArtifactIdSha256) -> c_int {
    let output = catch_panic(|| {
        check_null(ptr, Error::ArtifactIdPtrIsNull)?;
        let artifact_id = unsafe { &*ptr };
        let artifact_id: ArtifactId<Sha256> = const_transmute(artifact_id);
        let len = artifact_id.hash_len();
        Ok(len as c_int)
    });

    output.unwrap_or(Status::InvalidPtr as c_int)
}

/// Get the hash from a `ArtifactId` as an array of bytes.
///
/// # Safety
///
/// The ArtifactId pointer should not be null.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_hash_bytes(ptr: *const ArtifactIdSha256) -> *const u8 {
    let output = catch_panic(|| {
        check_null(ptr, Error::ArtifactIdPtrIsNull)?;
        let artifact_id = unsafe { &*ptr };
        let artifact_id: ArtifactId<Sha256> = const_transmute(artifact_id);
        let hash = artifact_id.as_bytes();
        Ok(hash.as_ptr())
    });

    output.unwrap_or_else(null)
}

/// Get the hash from a `ArtifactId` as a C-string.
///
/// # Safety
///
/// Note that the returned string must be freed with a call to
/// `ob_str_free`.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_hash_string(ptr: *const ArtifactIdSha256) -> *mut c_char {
    let output = catch_panic(|| {
        check_null(ptr, Error::ArtifactIdPtrIsNull)?;
        let artifact_id = unsafe { &*ptr };
        let artifact_id: ArtifactId<Sha256> = const_transmute(artifact_id);
        let hash_str = artifact_id.as_hex();
        let hash_c_str = CString::new(hash_str)?;
        Ok(hash_c_str.into_raw())
    });

    output.unwrap_or_else(null_mut)
}

/// Get the name of a hash algorithm as a C-string which must be freed.
///
/// # Safety
///
/// Returns a null pointer if the string cannot be returned.
///
/// The returned string must be freed with `ob_str_free`.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_hash_algorithm(
    ptr: *const ArtifactIdSha256,
) -> *const c_char {
    let output = catch_panic(|| {
        check_null(ptr, Error::ArtifactIdPtrIsNull)?;
        let artifact_id = unsafe { &*ptr };
        let artifact_id: ArtifactId<Sha256> = const_transmute(artifact_id);
        let name = artifact_id.hash_algorithm();

        match name {
            "sha256" => Ok(HASH_ALGORITHM_SHA256),
            _ => unimplemented!(),
        }
    });

    output.unwrap_or_else(null)
}

/// Free the `ArtifactId` from memory.
///
/// # Safety
///
/// Does nothing if passed a null pointer.
#[no_mangle]
pub unsafe extern "C" fn ob_aid_sha256_free(ptr: *const ArtifactIdSha256) {
    if check_null(ptr, Error::ArtifactIdPtrIsNull).is_err() {
        return;
    }

    // SAFETY: This const-to-mut conversion is safe because the provenance was originally
    //         mut on creation in all constructors.
    let _ = unsafe { Box::from_raw(ptr as *mut ArtifactIdSha256) };

    // Dropped and freed automatically when the `Box` goes out of scope.
}
