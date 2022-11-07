use crate::HashAlgorithm;
use crate::ObjectType;
use crate::GitOid;
use std::slice;
use std::ffi::c_char;
use std::ffi::CStr;

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
    let c_str = unsafe {
        assert!(!s.is_null());
        CStr::from_ptr(s)
    };

    let s = c_str.to_str().unwrap();
    GitOid::new_from_str(hash_algorithm, object_type, s)
}