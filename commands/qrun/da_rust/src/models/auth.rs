//
// (C) Copyright IBM 2024
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct GetAccessTokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub expiration: Option<u64>,
    pub token_type: String,
    pub refresh_token: Option<String>,
}
