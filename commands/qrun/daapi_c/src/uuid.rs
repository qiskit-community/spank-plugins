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

use direct_access_api::utils::uuid;
use std::ffi::CString;
use std::os::raw::c_char;

/// @brief Generates a new random (V4) UUID
///
/// @return a new UUID v4 if succeeded, otherwise NULL. Must call daapi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub extern "C" fn daapi_uuid_v4_new() -> *const c_char {
    if let Ok(uuid) = CString::new(uuid::new_v4()) {
        return uuid.into_raw();
    }
    std::ptr::null()
}
