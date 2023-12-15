//! The main gitoid FFI functions.

use crate::ffi::error::catch_panic;
use crate::ffi::error::get_error_msg;
use crate::ffi::error::Error;
use crate::ffi::status::Status;
use crate::ffi::util::check_null;
use crate::ffi::util::write_to_c_buf;
use crate::GitOid;
use crate::HashAlgorithm;
use crate::ObjectType;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::CStr;
use core::ptr::null;
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use core::slice::from_raw_parts_mut;
use std::ffi::CString;
use std::fs::File;
use std::io::BufReader;
#[cfg(target_family = "unix")]
use std::os::unix::prelude::FromRawFd;
#[cfg(target_family = "unix")]
use std::os::unix::prelude::RawFd;
#[cfg(target_family = "windows")]
use std::os::windows::io::FromRawHandle;
#[cfg(target_family = "windows")]
use std::os::windows::prelude::RawHandle;
use url::Url;

/// Get the last-written error message written to a buffer.
///
/// The length passed must match the length of the buffer provided.
///
/// If the buffer pointer is null, the function will fail and return an
/// error code.
///
/// If successful, it returns the number of bytes written to the buffer.
#[no_mangle]
pub extern "C" fn gitoid_get_error_message(buffer: *mut c_char, length: c_int) -> c_int {
    // Make sure the buffer isn't null.
    if buffer.is_null() {
        return Status::BufferIsNull as c_int;
    }

    // Convert the buffer raw pointer into a byte slice.
    let buffer = unsafe { from_raw_parts_mut(buffer as *mut u8, length as usize) };

    // Get the last error, possibly empty if there isn't one.
    let last_err = get_error_msg().unwrap_or_default();

    // Try to write the error to the buffer.
    write_to_c_buf(&last_err, buffer)
}

/// Check if the given `GitOid` is invalid.
///
/// If it is invalid, it shouldn't be used, and no fields of it should be
/// taken to have any meaning.
#[no_mangle]
pub extern "C" fn gitoid_invalid(gitoid: *const GitOid) -> c_int {
    let output = catch_panic(|| {
        let gitoid = unsafe { &*gitoid };
        let result = gitoid.hash_len() == 0;
        Ok(result as c_int)
    });

    output.unwrap_or(Status::UnexpectedError as c_int)
}

/// Construct a new `GitOid` from a buffer of bytes.
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
pub extern "C" fn gitoid_new_from_bytes(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    content: *mut u8,
    content_len: usize,
) -> GitOid {
    let output = catch_panic(|| {
        check_null(content, Error::ContentPtrIsNull)?;
        let content = unsafe { from_raw_parts(content, content_len) };
        Ok(GitOid::new_from_bytes(hash_algorithm, object_type, content))
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

/// Construct a new `GitOid` from a C-string of data.
///
/// The string passed _must_ be a valid C-string with a nul-terminator at the
/// end, all contained in a single contiguous allocation. The pointer must also
/// not be null.
#[no_mangle]
pub extern "C" fn gitoid_new_from_str(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    s: *const c_char,
) -> GitOid {
    let output = catch_panic(|| {
        check_null(s, Error::StringPtrIsNull)?;
        let s = unsafe { CStr::from_ptr(s) }.to_str()?;
        Ok(GitOid::new_from_str(hash_algorithm, object_type, s))
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

/// Construct a new `GitOid` from a `URL` in a C-string.
#[no_mangle]
pub extern "C" fn gitoid_new_from_url(s: *const c_char) -> GitOid {
    let output = catch_panic(|| {
        check_null(s, Error::StringPtrIsNull)?;
        let raw_url = unsafe { CStr::from_ptr(s) }.to_str()?;
        let url = Url::parse(raw_url)?;
        let gitoid = GitOid::new_from_url(url)?;
        Ok(gitoid)
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

/// Create a new `GitOid` by reading data from a file.
///
/// The provided file descriptor must be valid and open for reading.
///
/// Returns an invalid `GitOid` if construction fails.
#[cfg(target_family = "unix")]
#[no_mangle]
pub extern "C" fn gitoid_new_from_reader(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    fd: RawFd,
) -> GitOid {
    let output = catch_panic(|| {
        let file = unsafe { File::from_raw_fd(fd) };
        let reader = BufReader::new(file);
        let gitoid = GitOid::new_from_reader(hash_algorithm, object_type, reader)?;
        Ok(gitoid)
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

/// Create a new `GitOid` by reading data from a file.
///
/// The provided file handle must be valid and open for reading.
///
/// Returns an invalid `GitOid` if construction fails.
#[cfg(target_family = "windows")]
#[no_mangle]
/// cbindgen:ignore
pub extern "C" fn gitoid_new_from_reader(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    handle: RawHandle,
) -> GitOid {
    let output = catch_panic(|| {
        let file = unsafe { File::from_raw_handle(handle) };
        let reader = BufReader::new(file);
        let gitoid = GitOid::new_from_reader(hash_algorithm, object_type, reader)?;
        Ok(gitoid)
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

/// Construct a URL representation of a `GitOid`.
///
/// The resulting string _must_ be freed with a call to `gitoid_str_free`.
///
/// Returns a `NULL` pointer if the URL construction fails.
#[no_mangle]
pub extern "C" fn gitoid_get_url(ptr: *mut GitOid) -> *mut c_char {
    let output = catch_panic(|| {
        check_null(ptr, Error::GitOidPtrIsNull)?;
        let gitoid = unsafe { &mut *ptr };
        let url = CString::new(gitoid.url().as_str())?;
        Ok(url.into_raw())
    });

    output.unwrap_or_else(null_mut)
}

/// Get the hash from a `GitOid` as an array of bytes.
#[no_mangle]
pub extern "C" fn gitoid_get_hash_bytes(ptr: *mut GitOid) -> *const u8 {
    let output = catch_panic(|| {
        let gitoid = unsafe { &*ptr };
        let hash = gitoid.hash();
        Ok(hash.as_ptr())
    });

    output.unwrap_or_else(null)
}

/// Get the hash from a `GitOid` as a C-string.
///
/// Note that the returned string must be freed with a call to
/// `gitoid_str_free`.
#[no_mangle]
pub extern "C" fn gitoid_get_hash_string(ptr: *mut GitOid) -> *mut c_char {
    let output = catch_panic(|| {
        let gitoid = unsafe { &*ptr };
        let hash = gitoid.hash();
        let hash_str = hash.as_hex();
        let hash_c_str = CString::new(hash_str)?;
        Ok(hash_c_str.into_raw())
    });

    output.unwrap_or_else(null_mut)
}

/// Embed a C string into the binary.
macro_rules! embed_cstr {
    ($name:ident, $arr:expr) => {
        const $name: *const c_char = $arr.as_ptr();
    };
}

/// Get the name of a `HashAlgorithm` as a C-string.
///
/// Returns a null pointer if the string cannot be returned.
#[no_mangle]
pub extern "C" fn gitoid_hash_algorithm_name(hash_algorithm: HashAlgorithm) -> *const c_char {
    embed_cstr!(HASH_ALGORITHM_SHA1, [0x73, 0x68, 0x61, 0x31, 0x00]);
    embed_cstr!(
        HASH_ALGORITHM_SHA256,
        [0x73, 0x68, 0x61, 0x32, 0x35, 0x36, 0x00]
    );

    let output = catch_panic(|| {
        Ok(match hash_algorithm {
            HashAlgorithm::Sha1 => HASH_ALGORITHM_SHA1,
            HashAlgorithm::Sha256 => HASH_ALGORITHM_SHA256,
        })
    });

    output.unwrap_or_else(null)
}

/// Get the name of an `ObjectType` as a C-string.
///
/// Returns a null pointer if the string cannot be returned.
#[no_mangle]
pub extern "C" fn gitoid_object_type_name(object_type: ObjectType) -> *const c_char {
    embed_cstr!(OBJECT_TYPE_BLOB, [0x62, 0x6C, 0x6F, 0x62, 0x00]);
    embed_cstr!(OBJECT_TYPE_TREE, [0x74, 0x72, 0x65, 0x65, 0x00]);
    embed_cstr!(
        OBJECT_TYPE_COMMIT,
        [0x63, 0x6F, 0x6D, 0x6D, 0x69, 0x74, 0x00]
    );
    embed_cstr!(OBJECT_TYPE_TAG, [0x74, 0x61, 0x67, 0x00]);

    let output = catch_panic(|| {
        Ok(match object_type {
            ObjectType::Blob => OBJECT_TYPE_BLOB,
            ObjectType::Tree => OBJECT_TYPE_TREE,
            ObjectType::Commit => OBJECT_TYPE_COMMIT,
            ObjectType::Tag => OBJECT_TYPE_TAG,
        })
    });

    output.unwrap_or_else(null)
}

/// Free the given string.
///
/// Does nothing if the pointer is `NULL`.
///
/// This function must only ever be called with strings obtained from another
/// `gitoid` FFI function and where the function documentation indicates that
/// the string needs to be freed.
#[no_mangle]
pub extern "C" fn gitoid_str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    let _ = unsafe { CString::from_raw(s) };
}
