//
// (C) Copyright IBM 2024, 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use crate::Client;
use anyhow::{bail, Result};
use http::StatusCode;
use serde_json::json;

impl Client {
    /// Creates a new session using the specified mode and maximum TTL.
    ///
    /// Sends a POST request to the `/v1/sessions` endpoint with the provided parameters.
    /// On success, returns the session ID as a string.
    pub async fn get_session(&self, mode: &str, max_ttl: u64) -> Result<String> {
        let url = format!("{}/v1/sessions", self.base_url);
        let payload = json!({
            "mode": mode,
            "max_ttl": max_ttl
        });
        
        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send()
            .await?;
            
        let status_code = resp.status();
        if status_code == StatusCode::NO_CONTENT {
            bail!("No session created: no content returned");
        }
        
        let json_data = resp.json::<serde_json::Value>().await?;
        if let Some(session_id) = json_data["session_id"].as_str() {
            Ok(session_id.to_string())
        } else {
            bail!("Invalid session response: {}", json_data)
        }
    }
}
