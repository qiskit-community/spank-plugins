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

use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use crate::consts::{DAAPI_ERROR, DAAPI_SUCCESS};

/// @brief Free a string allocated by C API
///
/// # Safety
///
/// * The memory pointed to by `ptr` must contain a valid nul terminator at the
///   end of the string.
///
/// * `ptr` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `ptr` must be non-null even for a zero-length cstr.
///    
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `ptr`
///
/// @param (ptr) [in] pointer to the memory to be free
/// @return DAAPI_SUCCESS(0) if suceeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_string(ptr: *mut c_char) -> c_int {
    ffi_helpers::null_pointer_check!(ptr, DAAPI_ERROR);
    unsafe {
        drop(CString::from_raw(ptr));
    }
    DAAPI_SUCCESS
}
