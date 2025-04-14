/*
 * Qiskit Runtime API
 *
 * The Qiskit Runtime API description
 *
 * The version of the OpenAPI document: 0.21.2
 *
 * Generated by: https://openapi-generator.tech
 */

use super::{configuration, ContentType, Error};
use crate::{apis::ResponseContent, models};
use reqwest;
use serde::{de::Error as _, Deserialize, Serialize};

/// struct for typed errors of method [`create_session`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateSessionError {
    Status400(models::ListJobs400Response),
    Status401(models::ListJobs400Response),
    Status500(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_session_close`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteSessionCloseError {
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_session_information`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetSessionInformationError {
    Status401(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    Status500(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`update_session_state`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UpdateSessionStateError {
    Status401(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    Status500(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// Create a session
pub async fn create_session(
    configuration: &configuration::Configuration,
    ibm_api_version: Option<&str>,
    create_session_request: Option<models::CreateSessionRequest>,
) -> Result<models::CreateSession200Response, Error<CreateSessionError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_ibm_api_version = ibm_api_version;
    let p_create_session_request = create_session_request;

    let uri_str = format!("{}/sessions", configuration.base_path);
    let mut req_builder = configuration
        .client
        .request(reqwest::Method::POST, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = p_ibm_api_version {
        req_builder = req_builder.header("IBM-API-Version", param_value.to_string());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    if let Some(ref crn) = configuration.crn {
        req_builder = req_builder.header("Service-CRN", crn.clone());
    }
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    req_builder = req_builder.json(&p_create_session_request);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::CreateSession200Response`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::CreateSession200Response`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<CreateSessionError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Closes the runtime session
pub async fn delete_session_close(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<(), Error<DeleteSessionCloseError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/sessions/{id}/close",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration
        .client
        .request(reqwest::Method::DELETE, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = p_ibm_api_version {
        req_builder = req_builder.header("IBM-API-Version", param_value.to_string());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    if let Some(ref crn) = configuration.crn {
        req_builder = req_builder.header("Service-CRN", crn.clone());
    }
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<DeleteSessionCloseError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Get a session
pub async fn get_session_information(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<models::CreateSession200Response, Error<GetSessionInformationError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/sessions/{id}",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = p_ibm_api_version {
        req_builder = req_builder.header("IBM-API-Version", param_value.to_string());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    if let Some(ref crn) = configuration.crn {
        req_builder = req_builder.header("Service-CRN", crn.clone());
    }
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");
    let content_type = super::ContentType::from(content_type);

    if !status.is_client_error() && !status.is_server_error() {
        let content = resp.text().await?;
        match content_type {
            ContentType::Json => serde_json::from_str(&content).map_err(Error::from),
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::CreateSession200Response`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::CreateSession200Response`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetSessionInformationError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Update a session
pub async fn update_session_state(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
    update_session_state_request: Option<models::UpdateSessionStateRequest>,
) -> Result<(), Error<UpdateSessionStateError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;
    let p_update_session_state_request = update_session_state_request;

    let uri_str = format!(
        "{}/sessions/{id}",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration
        .client
        .request(reqwest::Method::PATCH, &uri_str);

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = p_ibm_api_version {
        req_builder = req_builder.header("IBM-API-Version", param_value.to_string());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };

    if let Some(ref crn) = configuration.crn {
        req_builder = req_builder.header("Service-CRN", crn.clone());
    }
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    req_builder = req_builder.json(&p_update_session_state_request);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<UpdateSessionStateError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}
