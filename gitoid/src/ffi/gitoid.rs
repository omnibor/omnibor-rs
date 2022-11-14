use crate::HashAlgorithm;
use crate::ObjectType;
use crate::GitOid;
use std::slice;
use std::ffi::c_char;
use std::ffi::CStr;
use url::Url;

#[no_mangle]
pub extern fn new_from_bytes(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    content: *const u8,
    content_len: usize,
) -> GitOid {

    // TODO: Make sure that content_len is less than or equal to isize::MAX.
    let content = unsafe { slice::from_raw_parts(content, content_len) };
    GitOid::new_from_bytes(hash_algorithm, object_type, content)
}

#[no_mangle]
pub extern fn new_from_str(
    hash_algorithm: HashAlgorithm,
    object_type: ObjectType,
    s: *const c_char,
) -> GitOid {
    // Based heavily on http://jakegoulding.com/rust-ffi-omnibus/string_arguments/
    // TODO: Make sure that content_len is less than or equal to isize::MAX.
    // TODO: Make sure that s is not nul-terminted.
    let c_str = unsafe {
        assert!(!s.is_null());
        CStr::from_ptr(s)
    };

    let s = c_str.to_str().unwrap();
    GitOid::new_from_str(hash_algorithm, object_type, s)
}

#[no_mangle]
pub extern fn new_from_url(s: *const c_char) -> GitOid {
    let c_str = unsafe {
        assert!(!s.is_null());
        CStr::from_ptr(s)
    };

    let s = c_str.to_str().unwrap();
    let url = Url::parse(s).unwrap();
    GitOid::new_from_url(url.clone()).unwrap()
}
 