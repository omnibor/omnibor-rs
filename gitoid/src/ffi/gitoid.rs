use crate::HashAlgorithm;
use crate::ObjectType;
use crate::GitOid;
use std::slice;
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
    // Based off https://stackoverflow.com/a/24148033/2308264

    // TODO: Also make sure that content_len is less than or equal to isize::MAX.
    // CStr::from_ptr calls slice::from_raw_parts
    // Are there other safety checks we should do here?
    // https://doc.rust-lang.org/std/ffi/struct.CStr.html#method.from_ptr
    let content_string: &CStr = unsafe { CStr::from_ptr(s) };
    let content = content_string.as_bytes();
    GitOid::new_from_str(hash_algorithm, object_type, content)
}