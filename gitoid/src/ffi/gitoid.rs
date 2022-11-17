use crate::HashAlgorithm;
use crate::ObjectType;
use crate::GitOid;
use std::slice;
use std::ptr::null_mut;
use std::ffi::c_char;
use std::ffi::CStr;
use std::ffi::CString;
use url::Url;
use std::error::Error;
use std::ops::Not as _;

/// Construct a new `GitOid` from a buffer of bytes.
#[no_mangle]
pub extern fn gitoid_new_from_bytes(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    content: *const u8,
    content_len: usize,
) -> GitOid {
    // TODO: Make sure that content_len is less than or equal to isize::MAX.
    let content = unsafe { slice::from_raw_parts(content, content_len) };
    GitOid::new_from_bytes(hash_algorithm, object_type, content)
}

/// Construct a new `GitOid` from a C-string.
#[no_mangle]
pub extern fn gitoid_new_from_str(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    s: *const c_char,
) -> GitOid {
    // Based heavily on http://jakegoulding.com/rust-ffi-omnibus/string_arguments/
    assert!(s.is_null().not());

    // TODO: Make sure that s is not nul-terminated.
    let s = unsafe { CStr::from_ptr(s) }.to_str().unwrap();
    GitOid::new_from_str(hash_algorithm, object_type, s)
}

/// Construct a new `GitOid` from a `URL`.
#[no_mangle]
pub extern fn gitoid_new_from_url(s: *const c_char) -> GitOid {
    fn inner(s: *const c_char) -> Result<GitOid, Box<dyn Error>> {
        assert!(s.is_null().not());

        // TODO: Make sure that s is not nul-terminated.
        let raw_url = unsafe { CStr::from_ptr(s) }.to_str()?;
        let url = Url::parse(raw_url)?;
        let gitoid = GitOid::new_from_url(url.clone())?;
        Ok(gitoid)
    }

    match inner(s) {
        Ok(g) => g,
        Err(e) => panic!("{}", e),
    }
}

// TODO: gitoid_new_from_reader

/// Construct a URL representation of the `GitOid`.
///
/// The resulting string _must_ be freed with a call to `gitoid_str_free`.
///
/// Returns a `NULL` pointer if the URL construction fails.
#[no_mangle]
pub extern fn gitoid_get_url(ptr: *mut GitOid) -> *mut c_char {
    fn inner(ptr: *mut GitOid) -> Result<*mut c_char, Box<dyn Error>> {
        assert!(ptr.is_null().not());
        let gitoid = unsafe { &mut *ptr };
        let gitoid_url = gitoid.url()?;
        let s = CString::new(gitoid_url.as_str())?.into_raw();
        Ok(s)
    }

    match inner(ptr) {
        Ok(s) => s,
        Err(_) => null_mut(),
    }
}
 
// TODO: gitoid_hash

/// Get the name of a `HashAlgorithm` as a C-string.
///
/// The resulting string _must_ be freed with a call to `gitoid_str_free`.
///
/// Returns a `NULL` pointer if the C-string can't be constructed.
#[no_mangle]
pub extern fn gitoid_hash_algorithm_name(hash_algorithm: HashAlgorithm) -> *mut c_char {
    match CString::new(hash_algorithm.to_string()) {
        Ok(s) => s.into_raw(),
        Err(_) => null_mut(),
    }
}

/// Get the name of an `ObjectType` as a C-string.
///
/// The resulting string _must_ be freed with a call to `gitoid_str_free`.
///
/// Returns a `NULL` pointer if the C-string can't be constructed.
#[no_mangle]
pub extern fn gitoid_object_type_name(object_type: ObjectType) -> *mut c_char {
    match CString::new(object_type.to_string()) {
        Ok(s) => s.into_raw(),
        Err(_) => null_mut(),
    }
}

/// Free the given string.
///
/// Does nothing if the pointer is `NULL`.
#[no_mangle]
pub extern fn gitoid_str_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }

    unsafe { CString::from_raw(s) };
}
