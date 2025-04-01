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
use serde_json::{json, Value};

impl Client {
    /// Creates a new session with optional parameters.
    ///
    /// Sends a POST request to the `/v1/sessions` endpoint.
    /// - `mode`: Optional mode string.
    /// - `backend`: Optional backend identifier.
    /// - `instance`: Optional instance identifier.
    /// - `max_time`: Optional maximum time value.
    /// - `channel`: If set to `"ibm_quantum"`, `max_time` is sent as `"max_session_ttl"`,
    ///   otherwise as `"max_ttl"`.
    ///
    /// Returns the JSON response from the server.
    pub async fn create_session(
        &self,
        mode: Option<&str>,
        backend: Option<&str>,
        instance: Option<&str>,
        max_time: Option<u64>,
        channel: Option<&str>,
    ) -> Result<Value> {
        let url = format!("{}/v1/sessions", self.base_url);
        let mut payload = serde_json::Map::new();

        if let Some(m) = mode {
            payload.insert("mode".to_string(), json!(m));
        }
        if let Some(b) = backend {
            payload.insert("backend".to_string(), json!(b));
        }
        if let Some(i) = instance {
            payload.insert("instance".to_string(), json!(i));
        }
        if let Some(max) = max_time {
            if let Some(ch) = channel {
                if ch == "ibm_quantum" {
                    payload.insert("max_session_ttl".to_string(), json!(max));
                } else {
                    payload.insert("max_ttl".to_string(), json!(max));
                }
            } else {
                payload.insert("max_ttl".to_string(), json!(max));
            }
        }

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(Value::Object(payload).to_string())
            .send()
            .await?;

        if resp.status() == StatusCode::NO_CONTENT {
            bail!("No session created: no content returned");
        }

        let json_data = resp.json::<Value>().await?;
        Ok(json_data)
    }

    /// Cancels an active session.
    ///
    /// Sends a DELETE request to `/v1/sessions/{session_id}/close`.
    /// Returns `Ok(())` if the cancellation succeeds.
    pub async fn cancel_session(&self, session_id: &str) -> Result<()> {
        let url = format!("{}/v1/sessions/{}/close", self.base_url, session_id);
        let resp = self.client.delete(url).send().await?;
        if !resp.status().is_success() {
            bail!("Failed to cancel session: {}", resp.status());
        }
        Ok(())
    }

    /// Closes an active session.
    ///
    /// Sends a PATCH request to `/v1/sessions/{session_id}` with the payload
    /// `{"accepting_jobs": false}`. This tells the backend that no new jobs
    /// should be accepted while allowing queued or running jobs to complete.
    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let url = format!("{}/v1/sessions/{}", self.base_url, session_id);
        let payload = json!({ "accepting_jobs": false });
        let resp = self
            .client
            .patch(url)
            .header("Content-Type", "application/json")
            .body(payload.to_string())
            .send()
            .await?;
        if !resp.status().is_success() {
            bail!("Failed to close session: {}", resp.status());
        }
        Ok(())
    }

    /// Retrieves details for the specified session.
    ///
    /// Sends a GET request to `/v1/sessions/{session_id}` and returns the JSON
    /// response as a `serde_json::Value`.
    pub async fn session_details(&self, session_id: &str) -> Result<Value> {
        let url = format!("{}/v1/sessions/{}", self.base_url, session_id);
        let resp = self.client.get(url).send().await?;
        if !resp.status().is_success() {
            bail!("Failed to get session details: {}", resp.status());
        }
        let json_data = resp.json::<Value>().await?;
        Ok(json_data)
    }
}
