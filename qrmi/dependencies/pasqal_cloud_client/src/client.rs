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

//! Pasqal Cloud API Client

use anyhow::{bail, Result};

use log::{debug, info};
use reqwest::header;
use reqwest_middleware::ClientBuilder as ReqwestClientBuilder;
use serde::de::DeserializeOwned;

/// An asynchronous `Client` to make Requests with.
#[derive(Debug, Clone)]
pub struct Client {
    /// The base URL this client sends requests to
    pub(crate) base_url: String,
    /// HTTP client to interact with Direct Access API service
    pub(crate) client: reqwest_middleware::ClientWithMiddleware,
    pub(crate) project_id: String,
    pub(crate) token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetAuthInfoResponse {
    // The IAM access token that can be used to invoke Direct Access API. Use this token with the prefix Bearer in the HTTP header Authorization for invocations of Direct Access API.
    pub code: u64,
}


use serde::{Deserialize};

impl Client {

    pub async fn get_auth_info(&self) -> Result<String> {
        let text = self.get(&format!("{}/account/api/v1/auth/info", self.base_url)).await?;
        Ok(text)
    }

    pub(crate) async fn get(&self, url: &str) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION, 
                format!("Bearer {}", self.token)
            )
            .send()
            .await?;
        if resp.status().is_success() {
            let json_text = resp.text().await?;
            info!("{}", json_text);
            Ok(json_text)
        } else {
            let json_text = resp.text().await?;
            bail!("Fail {}", json_text);
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
    /// use direct_access_api::ClientBuilder;
    ///
    /// let _builder = ClientBuilder::new(token);
    /// ```
    pub fn new(token: String, project_id: String) -> Self {
        Self {
            base_url: "https://apis.pasqal.cloud".to_string(),
            token: token,
            project_id: project_id,
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
        headers.insert(reqwest::header::CONTENT_TYPE, reqwest::header::HeaderValue::from_static("application/json"));
        headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("Bearer {}", self.token)).unwrap());
        reqwest_client_builder = reqwest_client_builder.default_headers(headers);
        let reqwest_builder = ReqwestClientBuilder::new(reqwest_client_builder.build()?);

        Ok(Client {
            base_url: self.base_url.clone(),
            client: reqwest_builder.build(),
            project_id: self.project_id.clone(),
            token: self.token.clone(),
        })
    }
}