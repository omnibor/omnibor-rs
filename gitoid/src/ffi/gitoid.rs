use crate::ffi::error::catch_panic;
use crate::ffi::error::get_error_msg;
use crate::ffi::error::Error;
use crate::GitOid;
use crate::HashAlgorithm;
use crate::ObjectType;
use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_uint;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Write as _;
use std::ptr::null_mut;
use std::slice;
use url::Url;

#[no_mangle]
pub extern "C" fn gitoid_get_error_message(buffer: *mut c_char, length: c_int) -> c_int {
    // Make sure the buffer isn't null.
    if buffer.is_null() {
        return -1;
    }

    // Convert the buffer raw pointer into a byte slice.
    let buffer = unsafe { slice::from_raw_parts_mut(buffer as *mut u8, length as usize) };

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

    output.unwrap_or(-1)
}

/// Construct a new `GitOid` from a buffer of bytes.
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
    content: *const u8,
    content_len: usize,
) -> GitOid {
    let output = catch_panic(|| {
        if content.is_null() {
            return Err(Error::ContentPtrIsNull);
        }

        let content = unsafe { slice::from_raw_parts(content, content_len) };
        Ok(GitOid::new_from_bytes(hash_algorithm, object_type, content))
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

/// Construct a new `GitOid` from a C-string.
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
        if s.is_null() {
            return Err(Error::StringPtrIsNull);
        }

        let s = match unsafe { CStr::from_ptr(s) }.to_str() {
            Ok(s) => s,
            Err(_) => return Err(Error::NotValidUtf8),
        };

        Ok(GitOid::new_from_str(hash_algorithm, object_type, s))
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

/// Construct a new `GitOid` from a `URL`.
#[no_mangle]
pub extern "C" fn gitoid_new_from_url(s: *const c_char) -> GitOid {
    let output = catch_panic(|| {
        if s.is_null() {
            return Err(Error::StringPtrIsNull);
        }

        let raw_url = match unsafe { CStr::from_ptr(s) }.to_str() {
            Ok(u) => u,
            Err(_) => return Err(Error::NotValidUtf8),
        };

        let url = match Url::parse(raw_url) {
            Ok(u) => u,
            Err(_) => return Err(Error::NotValidUrl),
        };

        let gitoid = match GitOid::new_from_url(url) {
            Ok(g) => g,
            Err(_) => return Err(Error::NotGitOidUrl),
        };

        Ok(gitoid)
    });

    output.unwrap_or_else(GitOid::new_invalid)
}

// TODO: gitoid_new_from_reader

/// Construct a URL representation of the `GitOid`.
///
/// The resulting string _must_ be freed with a call to `gitoid_str_free`.
///
/// Returns a `NULL` pointer if the URL construction fails.
#[no_mangle]
pub extern "C" fn gitoid_get_url(ptr: *mut GitOid) -> *mut c_char {
    let output = catch_panic(|| {
        if ptr.is_null() {
            return Err(Error::GitOidPtrIsNull);
        }

        let gitoid = unsafe { &mut *ptr };

        let url = match gitoid.url() {
            Ok(u) => u,
            Err(_) => return Err(Error::CouldNotConstructUrl),
        };

        let c_url = match CString::new(url.as_str()) {
            Ok(s) => s,
            Err(_) => return Err(Error::StringHadInteriorNul),
        };

        Ok(c_url.into_raw())
    });

    output.unwrap_or_else(null_mut)
}

// TODO: gitoid_hash

macro_rules! embed_cstr {
    ($name:ident, $arr:expr) => {
        const $name: *const c_char = $arr.as_ptr();
    };
}

/// Get the name of a `HashAlgorithm` as a C-string.
#[no_mangle]
pub extern "C" fn gitoid_hash_algorithm_name(hash_algorithm: HashAlgorithm) -> *const c_char {
    embed_cstr!(HASH_ALGORITHM_SHA1, [0x73, 0x68, 0x61, 0x31, 0x00]);
    embed_cstr!(
        HASH_ALGORITHM_SHA256,
        [0x73, 0x68, 0x61, 0x32, 0x35, 0x36, 0x00]
    );

    match hash_algorithm {
        HashAlgorithm::Sha1 => HASH_ALGORITHM_SHA1,
        HashAlgorithm::Sha256 => HASH_ALGORITHM_SHA256,
    }
}

/// Get the name of an `ObjectType` as a C-string.
#[no_mangle]
pub extern "C" fn gitoid_object_type_name(object_type: ObjectType) -> *const c_char {
    embed_cstr!(OBJECT_TYPE_BLOB, [0x62, 0x6C, 0x6F, 0x62, 0x00]);
    embed_cstr!(OBJECT_TYPE_TREE, [0x74, 0x72, 0x65, 0x65, 0x00]);
    embed_cstr!(
        OBJECT_TYPE_COMMIT,
        [0x63, 0x6F, 0x6D, 0x6D, 0x69, 0x74, 0x00]
    );
    embed_cstr!(OBJECT_TYPE_TAG, [0x74, 0x61, 0x67, 0x00]);

    match object_type {
        ObjectType::Blob => OBJECT_TYPE_BLOB,
        ObjectType::Tree => OBJECT_TYPE_TREE,
        ObjectType::Commit => OBJECT_TYPE_COMMIT,
        ObjectType::Tag => OBJECT_TYPE_TAG,
    }
}

/// Free the given string.
///
/// Does nothing if the pointer is `NULL`.
#[no_mangle]
pub extern "C" fn gitoid_str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    unsafe { CString::from_raw(s) };
}

/// Write a string slice to a C buffer safely.
///
/// This performs a write, including the null terminator and performing zeroization of any
/// excess in the destination buffer.
pub(crate) fn write_to_c_buf(src: &str, mut dst: &mut [u8]) -> c_int {
    // Ensure the string has the null terminator.
    let src = match CString::new(src.as_bytes()) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let src = src.as_bytes_with_nul();

    // Make sure the destination buffer is big enough.
    if dst.len() < src.len() {
        return -2;
    }

    // Write the buffer.
    match dst.write_all(src) {
        Ok(()) => 0,
        Err(_) => -3,
    }
}
