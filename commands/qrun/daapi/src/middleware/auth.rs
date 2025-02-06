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

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use http::Extensions;
#[allow(unused_imports)]
use log::{debug, error, info};
use reqwest::{header::HeaderValue, Client, Request, Response};
use reqwest_middleware::{Middleware, Next};
#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use crate::models::{auth::GetAccessTokenResponse, errors::ErrorResponse};

pub(crate) struct TokenManager {
    access_token: Option<String>,
    token_expiry: Option<Instant>,
    client: Client,
    base_url: String,
    authorization: String,
}
impl TokenManager {
    pub(crate) fn new(base_url: impl Into<String>, authorization: String) -> Self {
        Self {
            access_token: None,
            token_expiry: None,
            client: Client::new(),
            base_url: base_url.into(),
            authorization,
        }
    }
    async fn get_access_token(&mut self) -> Result<()> {
        let token_url = format!("{}/v1/token", self.base_url);
        let response = self
            .client
            .post(&token_url)
            .header(reqwest::header::AUTHORIZATION, self.authorization.clone())
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .send()
            .await?;
        if response.status().is_success() {
            let token_response: GetAccessTokenResponse = response.json().await?;
            self.access_token = Some(token_response.access_token);
            self.token_expiry =
                Some(Instant::now() + Duration::from_secs(token_response.expires_in));
        } else {
            let error_response = response.json::<ErrorResponse>().await?;
            bail!(format!(
                "{} ({}) {:?}",
                error_response.title, error_response.status_code, error_response.errors
            ));
        }
        Ok(())
    }
    async fn ensure_token_validity(&mut self) -> Result<()> {
        if self.access_token.is_none()
            || self.token_expiry.unwrap_or_else(Instant::now) <= Instant::now()
        {
            self.get_access_token().await?;
        }
        Ok(())
    }
    async fn get_token(&mut self) -> Result<String> {
        self.ensure_token_validity().await?;
        Ok(self.access_token.clone().unwrap())
    }
}

#[derive(Clone)]
pub(crate) struct AuthMiddleware {
    token_manager: Arc<Mutex<TokenManager>>,
}
impl AuthMiddleware {
    pub(crate) fn new(token_manager: Arc<Mutex<TokenManager>>) -> Self {
        Self { token_manager }
    }
}
#[async_trait]
impl Middleware for AuthMiddleware {
    async fn handle(
        &self,
        request: Request,
        extensions: &mut Extensions,
        next: Next<'_>,
    ) -> reqwest_middleware::Result<Response> {
        let mut token_manager = self.token_manager.lock().await;
        let token = token_manager.get_token().await?;
        // add authentication header to the request
        let mut cloned_req = request.try_clone().unwrap();
        info!("current token {}", token);
        cloned_req.headers_mut().insert(
            reqwest::header::AUTHORIZATION,
            format!("Bearer {}", token).parse().unwrap(),
        );

        // send a request
        let response = next
            .clone()
            .run(cloned_req.try_clone().unwrap(), extensions)
            .await;

        // retry if token is expired.
        if response.is_err()
            || response.as_ref().unwrap().status() == reqwest::StatusCode::UNAUTHORIZED
        {
            info!("renew access token");
            token_manager.get_access_token().await?;
            let token = token_manager.get_token().await?;
            info!("new token {}", token);
            //let mut new_request = cloned_req.try_clone().unwrap();
            let mut new_request = request.try_clone().unwrap();
            new_request.headers_mut().insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", token).parse().unwrap(),
            );
            return next.clone().run(new_request, extensions).await;
        }
        response
    }
}
