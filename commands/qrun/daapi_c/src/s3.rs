//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use anyhow::Result;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uchar, c_ulong};
use std::slice;

use crate::consts::{DAAPI_ERROR, DAAPI_SUCCESS};

/// @brief S3 API client handle
pub struct S3Client {
    #[allow(dead_code)]
    internal: direct_access_api::utils::s3::S3Client,
}

/// @brief Byte buffer
#[repr(C)]
pub struct Buffer {
    /// ptr to bytes data
    data: *mut u8,
    /// size in bytes
    size: usize,
}

/// @brief Metadata of a S3 Object
#[repr(C)]
#[derive(Debug)]
pub struct S3Object {
    /// key name
    key: *mut c_char,
}
/// @brief A list of S3 Objects
#[repr(C)]
#[derive(Debug)]
pub struct S3ObjectList {
    /// Ptr to the first S3Object
    objects: *mut S3Object,
    /// Number of S3Object included in the list
    length: usize,
}

/// @brief Creates a new S3Client handle.
///
/// # Safety
///
/// * The memory pointed to by `endpoint_url`/`aws_access_key_id`/`aws_secret_access_key`/`s3_region` must contain a valid nul terminator at the
///   end of the string.
///
/// * `endpoint_url`/`aws_access_key_id`/`aws_secret_access_key`/`s3_region` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `endpoint_url`/`aws_access_key_id`/`aws_secret_access_key`/`s3_region` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `endpoint_url`/`aws_access_key_id`/`aws_secret_access_key`/`s3_region`
///
/// @param (endpoint_url) [in] S3 endpoint URL
/// @param (aws_access_key_id) [in] AWS Access Key ID
/// @param (aws_secret_access_key) [in] AWS Secret Access Key
/// @param (s3_region) [in] S3 region (e.g. "us-east-1")
/// @return a new S3Client handle. Must call daapi_free_s3client() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_new(
    endpoint_url: *const c_char,
    aws_access_key_id: *const c_char,
    aws_secret_access_key: *const c_char,
    s3_region: *const c_char,
) -> *mut S3Client {
    ffi_helpers::null_pointer_check!(endpoint_url, std::ptr::null_mut::<S3Client>());
    ffi_helpers::null_pointer_check!(aws_access_key_id, std::ptr::null_mut::<S3Client>());
    ffi_helpers::null_pointer_check!(aws_secret_access_key, std::ptr::null_mut::<S3Client>());
    ffi_helpers::null_pointer_check!(s3_region, std::ptr::null_mut::<S3Client>());
    if let (Ok(endpoint), Ok(key), Ok(secret), Ok(region)) = (
        CStr::from_ptr(endpoint_url).to_str(),
        CStr::from_ptr(aws_access_key_id).to_str(),
        CStr::from_ptr(aws_secret_access_key).to_str(),
        CStr::from_ptr(s3_region).to_str(),
    ) {
        let client = Box::new(S3Client {
            internal: direct_access_api::utils::s3::S3Client::new(endpoint, key, secret, region),
        });
        return Box::into_raw(client);
    }
    std::ptr::null_mut::<S3Client>()
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
#[tokio::main]
async unsafe fn _get_presigned_url_for_get(
    client: *mut S3Client,
    bucket: &str,
    key: &str,
    expires_in: u64,
) -> Result<String> {
    (*client)
        .internal
        .get_presigned_url_for_get(bucket, key, expires_in)
        .await
}
/// @brief Returns the presigned URL for GetObject operation against the specified key in the S3 bucket
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket`/`key` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket`/`key` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket`/`key` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`/`key`
///
/// @param (client) [in] a S3Client handle
/// @param (bucket) [in] S3 bucket name
/// @param (key) [in] S3 object key
/// @param (expires_in) [in] The total amount of time (in seconds) the presigned request should be valid for.
/// @return The presigned URL for GetObject if succeeded, otherwise NULL. Must call daapi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_get_presigned_url_for_get(
    client: *mut S3Client,
    bucket: *const c_char,
    key: *const c_char,
    expires_in: c_ulong,
) -> *const c_char {
    ffi_helpers::null_pointer_check!(bucket, std::ptr::null());
    ffi_helpers::null_pointer_check!(key, std::ptr::null());
    if let (Ok(bucket_name), Ok(key_name)) = (
        CStr::from_ptr(bucket).to_str(),
        CStr::from_ptr(key).to_str(),
    ) {
        if let Ok(presigned_url) =
            _get_presigned_url_for_get(client, bucket_name, key_name, expires_in)
        {
            if let Ok(url_c) = CString::new(presigned_url) {
                return url_c.into_raw();
            }
        }
    }
    std::ptr::null()
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
#[tokio::main]
async unsafe fn _get_presigned_url_for_put(
    client: *mut S3Client,
    bucket: &str,
    key: &str,
    expires_in: u64,
) -> Result<String> {
    (*client)
        .internal
        .get_presigned_url_for_put(bucket, key, expires_in)
        .await
}
/// @brief Returns the presigned URL for PutObject operation against the specified key in the S3 bucket
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket`/`key` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket`/`key` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket`/`key` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`/`key`
///
/// @param (client) [in] a S3Client handle
/// @param (bucket) [in] S3 bucket name
/// @param (key) [in] S3 object key
/// @param (expires_in) [in] The total amount of time (in seconds) the presigned request should be valid for.
/// @return The presigned URL for PutObject if succeded, otherwise NULL. Must call daapi_free_string() to free if no longer needed.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_get_presigned_url_for_put(
    client: *mut S3Client,
    bucket: *const c_char,
    key: *const c_char,
    expires_in: c_ulong,
) -> *const c_char {
    ffi_helpers::null_pointer_check!(bucket, std::ptr::null());
    ffi_helpers::null_pointer_check!(key, std::ptr::null());
    if let (Ok(bucket_name), Ok(key_name)) = (
        CStr::from_ptr(bucket).to_str(),
        CStr::from_ptr(key).to_str(),
    ) {
        if let Ok(presigned_url) =
            _get_presigned_url_for_put(client, bucket_name, key_name, expires_in)
        {
            if let Ok(url_c) = CString::new(presigned_url) {
                return url_c.into_raw();
            }
        }
    }
    std::ptr::null()
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
#[tokio::main]
async unsafe fn _delete_object(client: *mut S3Client, bucket: &str, key: &str) -> Result<()> {
    (*client).internal.delete_object(bucket, key).await
}

/// @brief Deletes an object from the specified S3 bucket.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket`/`key` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket`/`key` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket`/`key` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`/`key`
///
/// @param (client) [in] a S3Client handle
/// @param (bucket) [in] S3 bucket name
/// @param (key) [in] S3 object key
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_delete_object(
    client: *mut S3Client,
    bucket: *const c_char,
    key: *const c_char,
) -> c_int {
    ffi_helpers::null_pointer_check!(bucket, -1);
    ffi_helpers::null_pointer_check!(key, -1);
    if let (Ok(bucket_name), Ok(key_name)) = (
        CStr::from_ptr(bucket).to_str(),
        CStr::from_ptr(key).to_str(),
    ) {
        if let Ok(()) = _delete_object(client, bucket_name, key_name) {
            return DAAPI_SUCCESS;
        }
    }
    DAAPI_ERROR
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
#[tokio::main]
async unsafe fn _put_object(
    client: *mut S3Client,
    bucket: &str,
    key: &str,
    content: &[u8],
) -> Result<()> {
    (*client).internal.put_object(bucket, key, content).await
}
/// @brief Adds an object to the specified S3 bucket as string.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket`/`key`/`content` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket`/`key`/`content` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket`/`key`/`content` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`/`key`/`content`
///
/// @param (client) [in] a S3Client handle
/// @param (bucket) [in] S3 bucket name
/// @param (key) [in] S3 object key
/// @param (content) [in] null terminated string data
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_put_object_as_string(
    client: *mut S3Client,
    bucket: *const c_char,
    key: *const c_char,
    content: *const c_char,
) -> c_int {
    ffi_helpers::null_pointer_check!(bucket, -1);
    ffi_helpers::null_pointer_check!(key, -1);
    ffi_helpers::null_pointer_check!(content, -1);
    if let (Ok(bucket_name), Ok(key_name), content_bytes) = (
        CStr::from_ptr(bucket).to_str(),
        CStr::from_ptr(key).to_str(),
        CStr::from_ptr(content).to_bytes(),
    ) {
        if let Ok(()) = _put_object(client, bucket_name, key_name, content_bytes) {
            return DAAPI_SUCCESS;
        }
    }
    DAAPI_ERROR
}

/// @brief Adds an object to the specified S3 bucket as bytes.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket`/`key` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket`/`key` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket`/`key` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`/`key`
///
/// @param (client) [in] a S3Client handle
/// @param (bucket) [in] S3 bucket name
/// @param (key) [in] S3 object key
/// @param (data) [in] data ptr
/// @param (length) [in] data length
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_put_object_as_bytes(
    client: *mut S3Client,
    bucket: *const c_char,
    key: *const c_char,
    data: *const c_uchar,
    length: usize,
) -> c_int {
    ffi_helpers::null_pointer_check!(bucket, -1);
    ffi_helpers::null_pointer_check!(key, -1);
    ffi_helpers::null_pointer_check!(data, -1);
    if let (Ok(bucket_name), Ok(key_name)) = (
        CStr::from_ptr(bucket).to_str(),
        CStr::from_ptr(key).to_str(),
    ) {
        let bytes: &[u8] = slice::from_raw_parts(data, length);
        if let Ok(()) = _put_object(client, bucket_name, key_name, bytes) {
            return DAAPI_SUCCESS;
        }
    }
    DAAPI_ERROR
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
#[tokio::main]
async unsafe fn _get_object(client: *mut S3Client, bucket: &str, key: &str) -> Result<Vec<u8>> {
    (*client).internal.get_object(bucket, key).await
}

/// @brief Retrieves an object from the specified S3 bucket as string.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket`/`key` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket`/`key` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket`/`key` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`/`key`
///
/// @param (client) [in] a S3Client handle
/// @param (bucket) [in] S3 bucket name
/// @param (key) [in] S3 object key
/// @return a S3 object as string if succeeded, otherwise NULL. Must call daapi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_get_object_as_string(
    client: *mut S3Client,
    bucket: *const c_char,
    key: *const c_char,
) -> *const c_char {
    ffi_helpers::null_pointer_check!(bucket, std::ptr::null());
    ffi_helpers::null_pointer_check!(key, std::ptr::null());
    if let (Ok(bucket_name), Ok(key_name)) = (
        CStr::from_ptr(bucket).to_str(),
        CStr::from_ptr(key).to_str(),
    ) {
        if let Ok(data) = _get_object(client, bucket_name, key_name) {
            if let Ok(obj_as_str) = String::from_utf8(data) {
                if let Ok(obj) = CString::new(obj_as_str) {
                    return obj.into_raw();
                }
            }
        }
    }
    std::ptr::null()
}

/// @brief Retrieves an object from the specified S3 bucket as bytes.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket`/`key` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket`/`key` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket`/`key` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`/`key`
///
/// @param (client) [in] a S3Client handle
/// @param (bucket) [in] S3 bucket name
/// @param (key) [in] S3 object key
/// @return a Buffer handle if succeeded, otherwise NULL. Must call daapi_free_buffer() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_get_object_as_bytes(
    client: *mut S3Client,
    bucket: *const c_char,
    key: *const c_char,
) -> *mut Buffer {
    ffi_helpers::null_pointer_check!(bucket, std::ptr::null_mut::<Buffer>());
    ffi_helpers::null_pointer_check!(key, std::ptr::null_mut::<Buffer>());

    if let (Ok(bucket_name), Ok(key_name)) = (
        CStr::from_ptr(bucket).to_str(),
        CStr::from_ptr(key).to_str(),
    ) {
        if let Ok(mut data) = _get_object(client, bucket_name, key_name) {
            let buf = Box::new(Buffer {
                data: data.as_mut_ptr(),
                size: data.len(),
            });
            std::mem::forget(data);
            return Box::into_raw(buf);
        }
    }
    std::ptr::null_mut::<Buffer>()
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
#[tokio::main]
async unsafe fn _list_objects(client: *mut S3Client, bucket: &str) -> Result<Vec<String>> {
    (*client).internal.list_objects(bucket).await
}

/// @brief Lists object names available in the specified S3 bucket.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_s3cli_new().
///
/// * The memory pointed to by `bucket` must contain a valid nul terminator at the
///   end of the string.
///
/// * `bucket` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `bucket` must be non-null even for a zero-length cstr.
///     
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `bucket`
///
/// @param (bucket) [in] S3 bucket name
/// @return an S3ObjectList if succeeded, otherwise NULL. Must call daapi_free_s3_object_list() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_s3cli_list_objects(
    client: *mut S3Client,
    bucket: *const c_char,
) -> *mut S3ObjectList {
    ffi_helpers::null_pointer_check!(bucket, std::ptr::null_mut::<S3ObjectList>());
    if let Ok(bucket) = CStr::from_ptr(bucket).to_str() {
        if let Ok(result) = _list_objects(client, bucket) {
            let mut carray = Vec::new();
            for key in result {
                carray.push(S3Object {
                    key: CString::new(key).unwrap().into_raw(),
                });
            }
            let boxed_array = Box::new(S3ObjectList {
                objects: carray.as_mut_ptr(),
                length: carray.len(),
            });
            std::mem::forget(carray);
            return Box::into_raw(boxed_array);
        }
    }
    std::ptr::null_mut::<S3ObjectList>()
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_s3cli_list_objects(). Otherwise, or if `ptr` has already been freed, segmentation fault occurs.  If `ptr` is NULL, DAAPI_ERROR is returned.
///     
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_s3cli_list_objects().
///
/// @param (buf) [in] a S3ObjectList
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_s3_object_list(ptr: *mut S3ObjectList) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }

    unsafe {
        let array = Box::from_raw(ptr);

        for i in 0..array.length {
            let item = array.objects.add(i);
            if !(*item).key.is_null() {
                let _ = CString::from_raw((*item).key);
            }
        }
        let _ = Vec::from_raw_parts(array.objects, array.length, array.length);
    }
    DAAPI_SUCCESS
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_s3cli_get_object_as_bytes(). Otherwise, or if `ptr` has already been freed, segmentation fault occurs.  If `ptr` is NULL, DAAPI_ERROR is returned.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_s3cli_get_object_as_bytes().
///
/// @param (ptr) [in] a Buffer
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
extern "C" fn daapi_free_buffer(ptr: *mut Buffer) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }

    let s = unsafe { std::slice::from_raw_parts_mut((*ptr).data, (*ptr).size) };
    let s = s.as_mut_ptr();
    unsafe {
        let _ = Box::from_raw(s);
    }
    DAAPI_SUCCESS
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_s3cli_new(). Otherwise, or if `ptr` has already been freed, segmentation fault occurs.  If `ptr` is NULL, DAAPI_ERROR is returned.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_s3cli_new().
///
/// @param (ptr) [in] a S3Client
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_s3client(ptr: *mut S3Client) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }

    unsafe {
        let _ = Box::from_raw(ptr);
    };
    DAAPI_SUCCESS
}
