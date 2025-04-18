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

/// struct for typed errors of method [`cancel_job_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CancelJobJidError {
    Status400(models::ListJobs400Response),
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    Status409(models::ListJobs400Response),
    Status500(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`create_job`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CreateJobError {
    Status400(models::ListJobs400Response),
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    Status409(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`delete_job_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeleteJobJidError {
    Status400(models::ListJobs400Response),
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    Status500(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_interim_results_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetInterimResultsJidError {
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_job_details_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetJobDetailsJidError {
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_job_metrics_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetJobMetricsJidError {
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_job_results_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetJobResultsJidError {
    Status400(models::ListJobs400Response),
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_jog_logs_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetJogLogsJidError {
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`get_transpiled_circuits_jid`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetTranspiledCircuitsJidError {
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`list_jobs`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ListJobsError {
    Status400(models::ListJobs400Response),
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// struct for typed errors of method [`replace_job_tags`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ReplaceJobTagsError {
    Status401(models::ListJobs400Response),
    Status403(models::ListJobs400Response),
    Status404(models::ListJobs400Response),
    UnknownValue(serde_json::Value),
}

/// Cancels the specified job.
pub async fn cancel_job_jid(
    configuration: &configuration::Configuration,
    id: &str,
    parent_job_id: Option<&str>,
    ibm_api_version: Option<&str>,
) -> Result<(), Error<CancelJobJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_parent_job_id = parent_job_id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/jobs/{id}/cancel",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration
        .client
        .request(reqwest::Method::POST, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = p_parent_job_id {
        req_builder = req_builder.header("Parent-Job-Id", param_value.to_string());
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

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<CancelJobJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Invoke a Qiskit Runtime primitive. Note the returned job ID.  You will use it to check the job's status and review results. This request is rate limited to 5 jobs per minute per user.
pub async fn create_job(
    configuration: &configuration::Configuration,
    ibm_api_version: Option<&str>,
    parent_job_id: Option<&str>,
    create_job_request: Option<models::CreateJobRequest>,
) -> Result<models::CreateJob200Response, Error<CreateJobError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_ibm_api_version = ibm_api_version;
    let p_parent_job_id = parent_job_id;
    let p_create_job_request = create_job_request;

    let uri_str = format!("{}/jobs", configuration.base_path);
    let mut req_builder = configuration
        .client
        .request(reqwest::Method::POST, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    if let Some(ref user_agent) = configuration.user_agent {
        req_builder = req_builder.header(reqwest::header::USER_AGENT, user_agent.clone());
    }
    if let Some(param_value) = p_ibm_api_version {
        req_builder = req_builder.header("IBM-API-Version", param_value.to_string());
    }
    if let Some(param_value) = p_parent_job_id {
        req_builder = req_builder.header("Parent-Job-Id", param_value.to_string());
    }
    if let Some(ref token) = configuration.bearer_access_token {
        req_builder = req_builder.bearer_auth(token.to_owned());
    };
    if let Some(ref crn) = configuration.crn {
        req_builder = req_builder.header("Service-CRN", crn.clone());
    }
    req_builder = req_builder.json(&p_create_job_request);

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
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::CreateJob200Response`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::CreateJob200Response`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<CreateJobError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Delete the specified job and its associated data. Job must be in a terminal state.
pub async fn delete_job_jid(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<(), Error<DeleteJobJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/jobs/{id}",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration
        .client
        .request(reqwest::Method::DELETE, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

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

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<DeleteJobJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Return the interim results from this job. Interim results are kept two days after the job has finished running.
pub async fn get_interim_results_jid(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<String, Error<GetInterimResultsJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/jobs/{id}/interim_results",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

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
            ContentType::Text => Ok(content),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `String`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetInterimResultsJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// List the details about the specified quantum program job.
pub async fn get_job_details_jid(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
    exclude_params: Option<bool>,
) -> Result<models::JobResponse, Error<GetJobDetailsJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;
    let p_exclude_params = exclude_params;

    let uri_str = format!(
        "{}/jobs/{id}",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    if let Some(ref param_value) = p_exclude_params {
        req_builder = req_builder.query(&[("exclude_params", &param_value.to_string())]);
    }
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
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::JobResponse`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::JobResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetJobDetailsJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Gets metrics of specified job
pub async fn get_job_metrics_jid(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<models::JobMetrics, Error<GetJobMetricsJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/jobs/{id}/metrics",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

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
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::JobMetrics`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::JobMetrics`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetJobMetricsJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Return the final result from this job.
pub async fn get_job_results_jid(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<String, Error<GetJobResultsJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/jobs/{id}/results",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

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
            ContentType::Text => Ok(content),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `String`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetJobResultsJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// List all job logs for the specified job.
pub async fn get_jog_logs_jid(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<String, Error<GetJogLogsJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/jobs/{id}/logs",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

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
            ContentType::Text => Ok(content),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `String`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetJogLogsJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Return a presigned download URL for the transpiled circuits. Currently supported only for sampler primitive.
pub async fn get_transpiled_circuits_jid(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
) -> Result<models::JobsTranspiledCircuitsResponse, Error<GetTranspiledCircuitsJidError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;

    let uri_str = format!(
        "{}/jobs/{id}/transpiled_circuits",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

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
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::JobsTranspiledCircuitsResponse`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::JobsTranspiledCircuitsResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<GetTranspiledCircuitsJidError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// List the quantum program jobs you have run.
pub async fn list_jobs(
    configuration: &configuration::Configuration,
    ibm_api_version: Option<&str>,
    limit: Option<i32>,
    offset: Option<i32>,
    pending: Option<bool>,
    program: Option<&str>,
    backend: Option<&str>,
    created_after: Option<String>,
    created_before: Option<String>,
    sort: Option<&str>,
    tags: Option<Vec<String>>,
    session_id: Option<&str>,
    exclude_params: Option<bool>,
) -> Result<models::JobsResponse, Error<ListJobsError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_ibm_api_version = ibm_api_version;
    let p_limit = limit;
    let p_offset = offset;
    let p_pending = pending;
    let p_program = program;
    let p_backend = backend;
    let p_created_after = created_after;
    let p_created_before = created_before;
    let p_sort = sort;
    let p_tags = tags;
    let p_session_id = session_id;
    let p_exclude_params = exclude_params;

    let uri_str = format!("{}/jobs", configuration.base_path);
    let mut req_builder = configuration.client.request(reqwest::Method::GET, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

    if let Some(ref param_value) = p_limit {
        req_builder = req_builder.query(&[("limit", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_offset {
        req_builder = req_builder.query(&[("offset", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_pending {
        req_builder = req_builder.query(&[("pending", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_program {
        req_builder = req_builder.query(&[("program", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_backend {
        req_builder = req_builder.query(&[("backend", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_created_after {
        req_builder = req_builder.query(&[("created_after", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_created_before {
        req_builder = req_builder.query(&[("created_before", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_sort {
        req_builder = req_builder.query(&[("sort", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_tags {
        req_builder = match "multi" {
            "multi" => req_builder.query(
                &param_value
                    .iter()
                    .map(|p| ("tags".to_owned(), p.to_string()))
                    .collect::<Vec<(std::string::String, std::string::String)>>(),
            ),
            _ => req_builder.query(&[(
                "tags",
                &param_value
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
                    .to_string(),
            )]),
        };
    }
    if let Some(ref param_value) = p_session_id {
        req_builder = req_builder.query(&[("session_id", &param_value.to_string())]);
    }
    if let Some(ref param_value) = p_exclude_params {
        req_builder = req_builder.query(&[("exclude_params", &param_value.to_string())]);
    }
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
            ContentType::Text => Err(Error::from(serde_json::Error::custom("Received `text/plain` content type response that cannot be converted to `models::JobsResponse`"))),
            ContentType::Unsupported(unknown_type) => Err(Error::from(serde_json::Error::custom(format!("Received `{unknown_type}` content type response that cannot be converted to `models::JobsResponse`")))),
        }
    } else {
        let content = resp.text().await?;
        let entity: Option<ListJobsError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}

/// Replace job tags
pub async fn replace_job_tags(
    configuration: &configuration::Configuration,
    id: &str,
    ibm_api_version: Option<&str>,
    replace_job_tags_request: Option<models::ReplaceJobTagsRequest>,
) -> Result<(), Error<ReplaceJobTagsError>> {
    // add a prefix to parameters to efficiently prevent name collisions
    let p_id = id;
    let p_ibm_api_version = ibm_api_version;
    let p_replace_job_tags_request = replace_job_tags_request;

    let uri_str = format!(
        "{}/jobs/{id}/tags",
        configuration.base_path,
        id = crate::apis::urlencode(p_id)
    );
    let mut req_builder = configuration.client.request(reqwest::Method::PUT, &uri_str);
    req_builder = req_builder.header(reqwest::header::ACCEPT, "application/json");

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
    req_builder = req_builder.json(&p_replace_job_tags_request);

    let req = req_builder.build()?;
    let resp = configuration.client.execute(req).await?;

    let status = resp.status();

    if !status.is_client_error() && !status.is_server_error() {
        Ok(())
    } else {
        let content = resp.text().await?;
        let entity: Option<ReplaceJobTagsError> = serde_json::from_str(&content).ok();
        Err(Error::ResponseError(ResponseContent {
            status,
            content,
            entity,
        }))
    }
}
