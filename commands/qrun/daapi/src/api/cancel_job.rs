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

use crate::{Client, PrimitiveJob};
use anyhow::{bail, Result};
use http::StatusCode;

impl Client {
    /// Cancels the specified job if it has not yet terminated. Also deletes the job
    /// after cancellation if `delete_job` is set to `true`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use direct_access_api::{AuthMethod, ClientBuilder};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = ClientBuilder::new("http://localhost:8080")
    ///         .with_auth(AuthMethod::IbmCloudIam {
    ///             apikey: "your_iam_apikey".to_string(),
    ///             service_crn: "your_service_crn".to_string(),
    ///             iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///         })
    ///         .build()
    ///         .unwrap();
    ///     #[cfg(not(doctest))]
    ///     client.cancel_job("db4afb4a-2232-4b15-b750-3a327f05fc28", true).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - connection failed.
    /// - authentication failed.
    /// - specified job is not found.
    /// - a job has already terminated and cannot be cancelled.
    /// - an internal server error occurs.
    pub async fn cancel_job(&self, job_id: &str, delete_job: bool) -> Result<()> {
        let url = format!("{}/v1/jobs/{}/cancel", self.base_url, &job_id);
        let resp = self
            .client
            .post(url)
            .header("Content-Type", "application/json")
            .send()
            .await?;
        let status_code = resp.status();
        if status_code == StatusCode::NO_CONTENT {
            if !delete_job {
                return Ok(());
            }
            self.delete_job(job_id).await
        } else {
            let json_data = resp.json::<serde_json::Value>().await?;
            bail!(json_data.to_string())
        }
    }
}

impl PrimitiveJob {
    /// Cancels the specified job if it has not yet terminated. Also deletes the job
    /// after cancellation if `delete_job` is set to `true`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use serde_json::json;
    /// use direct_access_api::{AuthMethod, ClientBuilder, models::ProgramId, models::LogLevel};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let payload = json!({
    ///         "pubs":[
    ///             [
    ///                 "OPENQASM 3.0; include \\\"stdgates.inc\\\"; bit[2] meas; rz(pi/2) $0; sx $0; rz(pi/2) $0; cx $0, $1; meas[0] = measure $0; meas[1] = measure $1;",[],128
    ///             ],
    ///         ],
    ///         "supports_qiskit": false,
    ///         "version":2,
    ///     });
    ///
    ///     let client = ClientBuilder::new("http://localhost:8290")
    ///         .with_auth(AuthMethod::IbmCloudIam {
    ///             apikey: "your_iam_apikey".to_string(),
    ///             service_crn: "your_service_crn".to_string(),
    ///             iam_endpoint_url: "iam_endpoint_url".to_string(),
    ///         })
    ///         .with_s3bucket(
    ///             "my_aws_access_key_id",
    ///             "my_aws_secret_access_key",
    ///             "http://localhost:9000",
    ///             "my_bucket",
    ///             "us-east-1"
    ///         )
    ///         .build()
    ///         .unwrap();
    ///
    ///     let primitive_job = client
    ///         .run_primitive("ibm_brisbane", ProgramId::Sampler, 3600, LogLevel::Info, &payload)
    ///         .await?;
    ///     primitive_job.cancel(true).await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error variant when:
    /// - connection failed.
    /// - authentication failed.
    /// - specified job is not found.
    /// - a job has already terminated and cannot be cancelled.
    /// - an internal server error occurs.
    pub async fn cancel(&self, delete_job: bool) -> Result<()> {
        self.client.cancel_job(&self.job_id, delete_job).await
    }
}
