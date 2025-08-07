//
// (C) Copyright Pasqal SAS 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

//! Pasqal Cloud API Client

use anyhow::{bail, Result};

use crate::models::batch::BatchStatus;
use crate::models::device::DeviceType;
use log::info;
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An asynchronous `Client` to make Requests with.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub(crate) base_url: String,
    /// HTTP client to interact with Pasqal Cloud service
    pub(crate) client: reqwest_middleware::ClientWithMiddleware,
    pub(crate) project_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Response<T> {
    pub data: T,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDeviceResponseData {
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetDeviceSpecsResponseData {
    pub specs: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateBatchResponseData {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBatchResponseData {
    pub status: BatchStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CancelBatchResponseData {}

#[derive(Debug, Clone, Deserialize)]
pub struct GetBatchResultLinksResponseData {
    pub results_links: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub runs: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Batch {
    pub sequence_builder: String,
    pub jobs: Vec<Job>,
    pub device_type: String,
    pub project_id: String,
}

impl Client {
    pub async fn get_device(
        &self,
        device_type: DeviceType,
    ) -> Result<Response<GetDeviceResponseData>> {
        let url = format!(
            "{}/core-fast/api/v1/devices?device_type={}",
            self.base_url, device_type,
        );
        self.get(&url).await
    }
    /// Pasqal Cloud works with batches of jobs rather than
    /// individual jobs, see:
    /// https://docs.pasqal.com/cloud/batches/
    pub async fn create_batch(
        &self,
        sequence: String,
        job_runs: i32,
        device_type: DeviceType,
    ) -> Result<Response<CreateBatchResponseData>> {
        let url = format!("{}/core-fast/api/v1/batches", self.base_url);
        let batch = Batch {
            sequence_builder: sequence,
            jobs: Vec::from([Job { runs: job_runs }]),
            device_type: device_type.to_string(),
            project_id: self.project_id.clone(),
        };
        self.post(&url, batch).await
    }

    pub async fn cancel_batch(&self, batch_id: &str) -> Result<Response<CancelBatchResponseData>> {
        let url = format!(
            "{}/core-fast/api/v2/batches/{}/cancel",
            self.base_url, batch_id
        );
        self.patch(&url).await
    }

    pub async fn get_batch(&self, batch_id: &str) -> Result<Response<GetBatchResponseData>> {
        let url = format!("{}/core-fast/api/v2/batches/{}", self.base_url, batch_id);
        self.get(&url).await
    }

    pub async fn get_batch_results(&self, batch_id: &str) -> Result<String> {
        match self.get_batch_result_links(batch_id).await {
            Ok(resp) => {
                let results_links = resp.data.results_links;
                // by design only one job
                let mut results: String = "".to_string();
                for (i, (_job_id, result_link)) in results_links.iter().enumerate() {
                    if i > 0 {
                        bail!(format!(
                            "Unexpected multiple jobs in one Pasqal cloud batch"
                        ));
                    };
                    results = reqwest::get(result_link).await?.text().await?;
                }
                if results == *"" {
                    bail!(format!("No results found"));
                }
                Ok(results)
            }
            Err(_err) => Err(_err),
        }
    }

    async fn get_batch_result_links(
        &self,
        batch_id: &str,
    ) -> Result<Response<GetBatchResultLinksResponseData>> {
        let url = format!(
            "{}/core-fast/api/v1/batches/{}/results_link",
            self.base_url, batch_id
        );
        self.get(&url).await
    }

    pub async fn get_device_specs(
        &self,
        device_type: DeviceType,
    ) -> Result<Response<GetDeviceSpecsResponseData>> {
        let url = format!(
            "{}/core-fast/api/v1/devices/specs/{}",
            self.base_url, device_type
        );
        self.get(&url).await
    }

    pub(crate) async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.client.get(url).send().await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn patch<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let resp = self.client.patch(url).send().await?;
        self.handle_request(resp).await
    }

    pub(crate) async fn post<T: DeserializeOwned, U: Serialize>(
        &self,
        url: &str,
        body: U,
    ) -> Result<T> {
        let resp = self.client.post(url).json(&body).send().await?;
        self.handle_request(resp).await
    }

    async fn handle_request<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T> {
        if resp.status().is_success() {
            let json_text = resp.text().await?;
            info!("{}", json_text);
            let val = serde_json::from_str(&json_text)?;
            Ok(val)
        } else {
            let status = resp.status();
            let json_text = resp.text().await?;
            bail!("Status: {}, Fail {}", status, json_text);
        }
    }
}

/// A [`ClientBuilder`] can be used to create a [`Client`] with custom configuration.
#[must_use]
#[derive(Debug, Clone)]
pub struct ClientBuilder {
    /// The base URL this client sends requests to
    base_url: String,
    token: String,
    project_id: String,
}

impl ClientBuilder {
    /// Construct a new [`ClientBuilder`]
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_cloud_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new(token);
    /// ```
    pub fn new(token: String, project_id: String) -> Self {
        Self {
            base_url: "https://apis.pasqal.cloud".to_string(),
            token,
            project_id,
        }
    }

    /// Returns a [`Client`] that uses this [`ClientBuilder`] configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pasqal_cloud_api::{ClientBuilder, AuthMethod};
    ///
    /// let _builder = ClientBuilder::new()
    ///     .with_token("your_token".to_string())
    ///     .build()
    /// ```
    pub fn build(&mut self) -> Result<Client> {
        let mut reqwest_client_builder = reqwest::Client::builder();
        reqwest_client_builder = reqwest_client_builder.connection_verbose(true);

        let mut headers = header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap(),
        );
        reqwest_client_builder = reqwest_client_builder.default_headers(headers);
        let reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);

        Ok(Client {
            base_url: self.base_url.clone(),
            client: reqwest_builder.build(),
            project_id: self.project_id.clone(),
        })
    }
}
