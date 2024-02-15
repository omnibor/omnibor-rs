//! The main gitoid FFI functions.

use crate::ffi::error::catch_panic;
use crate::ffi::error::get_error_msg;
use crate::ffi::error::Error;
use crate::ffi::status::Status;
use crate::ffi::util::check_null;
use crate::ffi::util::write_to_c_buf;
use crate::object::Blob;
use crate::object::Commit;
use crate::object::Tag;
use crate::object::Tree;
use crate::GitOid;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::CStr;
use core::ptr::null;
use core::ptr::null_mut;
use core::slice::from_raw_parts;
use core::slice::from_raw_parts_mut;
use digest::OutputSizeUser;
use paste::paste;
use sha1::Sha1;
use sha1collisiondetection::Sha1CD as Sha1Cd;
use sha2::Sha256;
use std::ffi::CString;
use std::fs::File;
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
/// If successful, it returns the number of bytes written to the buffer.
///
/// # Safety
///
/// The length passed must match the length of the buffer provided.
///
/// If the buffer pointer is null, the function will fail and return an
/// error code.
#[no_mangle]
pub unsafe extern "C" fn gitoid_get_error_message(buffer: *mut c_char, length: c_int) -> c_int {
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
embed_cstr!(OBJECT_TYPE_TREE, [0x74, 0x72, 0x65, 0x65, 0x00]);
embed_cstr!(
    OBJECT_TYPE_COMMIT,
    [0x63, 0x6F, 0x6D, 0x6D, 0x69, 0x74, 0x00]
);
embed_cstr!(OBJECT_TYPE_TAG, [0x74, 0x61, 0x67, 0x00]);
embed_cstr!(HASH_ALGORITHM_SHA1, [0x73, 0x68, 0x61, 0x31, 0x00]);
embed_cstr!(
    HASH_ALGORITHM_SHA1DC,
    [0x73, 0x68, 0x61, 0x31, 0x64, 0x63, 0x00]
);
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
/// `gitoid` FFI function and where the function documentation indicates that
/// the string needs to be freed.
#[no_mangle]
pub unsafe extern "C" fn gitoid_str_free(s: *const c_char) {
    if s.is_null() {
        return;
    }

    let _ = unsafe { CString::from_raw(s as *mut _) };
}

// A helper macro to generate the FFI for an instantiation of the `GitOid`
// type with a specific hashing algorithm.
macro_rules! generate_gitoid_ffi_for_hash {
    ($hash_ty:ty, $hash_name:ident, $object_ty:ty, $object_name:ident) => {
        paste! {
            /// A `GitOid` constructed with the specified hash algorithm.
            pub struct [<GitOid $hash_ty $object_ty>](GitOid<$hash_ty, $object_ty>);

            /// Construct a new `GitOid` from a buffer of bytes.
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
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _from_bytes>](
                content: *mut u8,
                content_len: usize,
            ) -> *const [<GitOid $hash_ty $object_ty>] {
                let output = catch_panic(|| {
                    check_null(content, Error::ContentPtrIsNull)?;
                    let content = unsafe { from_raw_parts(content, content_len) };
                    let gitoid = GitOid::<$hash_ty, $object_ty>::from_bytes(content);
                    let boxed = Box::new(gitoid);
                    Ok(Box::into_raw(boxed) as *const _)
                });

                output.unwrap_or_else(null)
            }

            /// Construct a new `GitOid` from a C-string of data.
            ///
            /// # Safety
            ///
            /// The string passed _must_ be a valid C-string with a nul-terminator at the
            /// end, all contained in a single contiguous allocation. The pointer must also
            /// not be null.
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _from_str>](
                s: *const c_char,
            ) -> *const [<GitOid $hash_ty $object_ty>] {
                let output = catch_panic(|| {
                    check_null(s, Error::StringPtrIsNull)?;
                    let s = unsafe { CStr::from_ptr(s) }.to_str()?;
                    let gitoid = GitOid::<$hash_ty, $object_ty>::from_str(s);
                    let boxed = Box::new(gitoid);
                    Ok(Box::into_raw(boxed) as *const _)
                });

                output.unwrap_or_else(null)
            }

            /// Construct a new `GitOid` from a `URL` in a C-string.
            ///
            /// # Safety
            ///
            /// If the pointer is null, an error is returned.
            ///
            /// The returned `GitOid` must be freed.
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _from_url>](
                s: *const c_char
            ) -> *const [<GitOid $hash_ty $object_ty>] {
                let output = catch_panic(|| {
                    check_null(s, Error::StringPtrIsNull)?;
                    let raw_url = unsafe { CStr::from_ptr(s) }.to_str()?;
                    let url = Url::parse(raw_url)?;
                    let gitoid = GitOid::<$hash_ty, $object_ty>::from_url(url)?;
                    let boxed = Box::new(gitoid);
                    Ok(Box::into_raw(boxed) as *const _)
                });

                output.unwrap_or_else(null)
            }

            /// Create a new `GitOid` by reading data from a file.
            ///
            /// # Safety
            ///
            /// The provided file descriptor must be valid and open for reading.
            ///
            /// Returns an invalid `GitOid` if construction fails.
            #[cfg(target_family = "unix")]
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _from_reader>](
                fd: RawFd
            ) -> *const [<GitOid $hash_ty $object_ty>] {
                let output = catch_panic(|| {
                    let file = unsafe { File::from_raw_fd(fd) };
                    let gitoid = GitOid::<$hash_ty, $object_ty>::from_reader(file)?;
                    let boxed = Box::new(gitoid);
                    Ok(Box::into_raw(boxed) as *const _)
                });

                output.unwrap_or_else(null)
            }

            /// Create a new `GitOid` by reading data from a file.
            ///
            /// # Safety
            ///
            /// The provided file descriptor must be valid and open for reading.
            ///
            /// Returns an invalid `GitOid` if construction fails.
            #[cfg(target_family = "unix")]
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _from_reader_with_expected_length>](
                fd: RawFd,
                expected_length: c_int,
            ) -> *const [<GitOid $hash_ty $object_ty>] {
                let output = catch_panic(|| {
                    let file = unsafe { File::from_raw_fd(fd) };
                    let expected_length = expected_length as usize;
                    let gitoid = GitOid::<$hash_ty, $object_ty>::from_reader_with_expected_length(file, expected_length)?;
                    let boxed = Box::new(gitoid);
                    Ok(Box::into_raw(boxed) as *const _)
                });

                output.unwrap_or_else(null)
            }

            /// Create a new `GitOid` by reading data from a file.
            ///
            /// # Safety
            ///
            /// The provided file handle must be valid and open for reading.
            ///
            /// Returns an invalid `GitOid` if construction fails.
            #[cfg(target_family = "windows")]
            #[no_mangle]
            /// cbindgen:ignore
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _from_reader>](
                handle: RawHandle,
            ) -> *const [<GitOid $hash_ty $object_ty>] {
                let output = catch_panic(|| {
                    let file = unsafe { File::from_raw_handle(handle) };
                    let gitoid = GitOid::<$hash_ty, $object_ty>::from_reader(file)?;
                    let boxed = Box::new(gitoid);
                    Ok(Box::into_raw(boxed) as *const _)
                });

                output.unwrap_or_else(null)
            }

            /// Create a new `GitOid` by reading data from a file.
            ///
            /// # Safety
            ///
            /// The provided file handle must be valid and open for reading.
            ///
            /// Returns an invalid `GitOid` if construction fails.
            #[cfg(target_family = "windows")]
            #[no_mangle]
            /// cbindgen:ignore
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _from_reader_with_expected_length>](
                handle: RawHandle,
                expected_length: c_int,
            ) -> *const [<GitOid $hash_ty $object_ty>] {
                let output = catch_panic(|| {
                    let file = unsafe { File::from_raw_handle(handle) };
                    let expected_length = expected_length as usize;
                    let gitoid = GitOid::<$hash_ty, $object_ty>::from_reader_with_expected_length(file, expected_length)?;
                    let boxed = Box::new(gitoid);
                    Ok(Box::into_raw(boxed) as *const _)
                });

                output.unwrap_or_else(null)
            }

            /// Construct a URL representation of a `GitOid`.
            ///
            /// # Safety
            ///
            /// The resulting string _must_ be freed with a call to `gitoid_str_free`.
            ///
            /// Returns a `NULL` pointer if the URL construction fails.
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _get_url>](
                ptr: *const [<GitOid $hash_ty $object_ty>]
            ) -> *const c_char {
                let output = catch_panic(|| {
                    check_null(ptr, Error::GitOidPtrIsNull)?;
                    let gitoid = unsafe { &*ptr };
                    let url = CString::new(gitoid.0.url().as_str())?;
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
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _object_type_name>](
                ptr: *const [<GitOid $hash_ty $object_ty>]
            ) -> *const c_char {
                let output = catch_panic(|| {
                    check_null(ptr, Error::GitOidPtrIsNull)?;
                    let gitoid = unsafe { &* ptr };
                    let object_type = gitoid.0.object_type();

                    match object_type {
                        "blob" => Ok(OBJECT_TYPE_BLOB),
                        "commit" => Ok(OBJECT_TYPE_COMMIT),
                        "tag" => Ok(OBJECT_TYPE_TAG),
                        "tree" => Ok(OBJECT_TYPE_TREE),
                        _ => unimplemented!(),
                    }
                });

                output.unwrap_or_else(null)
            }

            /// Get the length of the `GitOid` hash in bytes.
            #[no_mangle]
            pub extern "C" fn [<gitoid_ $hash_name _ $object_name _hash_len>]() -> c_int {
                <$hash_ty as OutputSizeUser>::output_size() as c_int
            }

            /// Get the hash from a `GitOid` as an array of bytes.
            ///
            /// # Safety
            ///
            /// The gitoid pointer should not be null.
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _get_hash_bytes>](
                ptr: *const [<GitOid $hash_ty $object_ty>]
            ) -> *const u8 {
                let output = catch_panic(|| {
                    check_null(ptr, Error::GitOidPtrIsNull)?;
                    let gitoid = unsafe { &*ptr };
                    let hash = gitoid.0.as_bytes();
                    Ok(hash.as_ptr())
                });

                output.unwrap_or_else(null)
            }

            /// Get the hash from a `GitOid` as a C-string.
            ///
            /// # Safety
            ///
            /// Note that the returned string must be freed with a call to
            /// `gitoid_str_free`.
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _get_hash_string>](
                ptr: *const [<GitOid $hash_ty $object_ty>]
            ) -> *mut c_char {
                let output = catch_panic(|| {
                    check_null(ptr, Error::GitOidPtrIsNull)?;
                    let gitoid = unsafe { &*ptr };
                    let hash_str = gitoid.0.as_hex();
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
            /// The returned string must be freed with `gitoid_str_free`.
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _hash_algorithm_name>](
                ptr: *const [<GitOid $hash_ty $object_ty>]
            ) -> *const c_char {
                let output = catch_panic(|| {
                    check_null(ptr, Error::GitOidPtrIsNull)?;
                    let gitoid = unsafe { &* ptr };
                    let name = gitoid.0.hash_algorithm();

                    match name {
                        "sha1" => Ok(HASH_ALGORITHM_SHA1),
                        "sha1dc" => Ok(HASH_ALGORITHM_SHA1DC),
                        "sha256" => Ok(HASH_ALGORITHM_SHA256),
                        _ => unimplemented!(),
                    }
                });

                output.unwrap_or_else(null)
            }

            /// Free the `GitOid` from memory.
            ///
            /// # Safety
            ///
            /// Does nothing if passed a null pointer.
            #[no_mangle]
            pub unsafe extern "C" fn [<gitoid_ $hash_name _ $object_name _free>](
                ptr: *const [<GitOid $hash_ty $object_ty>]
            ) {
                if let Err(_) = check_null(ptr, Error::GitOidPtrIsNull) {
                    return;
                }

                // SAFETY: This const-to-mut conversion is safe because the provenance was originally
                //         mut on creation in all constructors.
                let _ = unsafe { Box::from_raw(ptr as *mut [<GitOid $hash_ty $object_ty>]) };

                // Dropped and freed automatically when the `Box` goes out of scope.
            }
        }
    };
}

generate_gitoid_ffi_for_hash!(Sha1, sha1, Blob, blob);
generate_gitoid_ffi_for_hash!(Sha1Cd, sha1cd, Blob, blob);
generate_gitoid_ffi_for_hash!(Sha256, sha256, Blob, blob);

generate_gitoid_ffi_for_hash!(Sha1, sha1, Commit, commit);
generate_gitoid_ffi_for_hash!(Sha1Cd, sha1cd, Commit, commit);
generate_gitoid_ffi_for_hash!(Sha256, sha256, Commit, commit);

generate_gitoid_ffi_for_hash!(Sha1, sha1, Tag, tag);
generate_gitoid_ffi_for_hash!(Sha1Cd, sha1cd, Tag, tag);
generate_gitoid_ffi_for_hash!(Sha256, sha256, Tag, tag);

generate_gitoid_ffi_for_hash!(Sha1, sha1, Tree, tree);
generate_gitoid_ffi_for_hash!(Sha1Cd, sha1cd, Tree, tree);
generate_gitoid_ffi_for_hash!(Sha256, sha256, Tree, tree);
