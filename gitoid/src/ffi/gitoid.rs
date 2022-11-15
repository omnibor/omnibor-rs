use crate::HashAlgorithm;
use crate::ObjectType;
use crate::GitOid;
use std::slice;
use std::ffi::c_char;
use std::ffi::CStr;
use std::ffi::CString;
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
    // TODO: Make sure that s is not nul-terminted.
    let c_str = unsafe {
        assert!(!s.is_null());
        CStr::from_ptr(s)
    };

    let s = c_str.to_str().unwrap();
    let url = Url::parse(s).unwrap();
    GitOid::new_from_url(url.clone()).unwrap()
}

// TODO: new_from_reader

#[no_mangle]
pub extern fn gitoid_url(ptr: *mut GitOid) -> *mut c_char {
    let gitoid = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };

    let gitoid_url = gitoid.url().unwrap();
    let gitoid_url_string = gitoid_url.as_str();

    let c_string = CString::new(gitoid_url_string).unwrap();
    c_string.into_raw()
}
 
// TO DO gitoid_hash

#[no_mangle]
pub extern fn gitoid_hash_algorithm(ptr: *mut GitOid) -> *mut c_char {
    // Returns string representation of the hash algorithm

    let gitoid = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };


    let hash_algorithm_string = format!("{}", gitoid.hash_algorithm());
    let c_string = CString::new(hash_algorithm_string).unwrap();
    c_string.into_raw()
}

#[no_mangle]
pub extern fn gitoid_object_type(ptr: *mut GitOid) -> *mut c_char {
    // Returns string representation of the object type

    let gitoid = unsafe {
        assert!(!ptr.is_null());
        &mut *ptr
    };

    let object_type_string = format!("{}", gitoid.object_type());
    let c_string = CString::new(object_type_string).unwrap();
    c_string.into_raw()
}