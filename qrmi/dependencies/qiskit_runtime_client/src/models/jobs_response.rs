/*
 * Qiskit Runtime API
 *
 * The Qiskit Runtime API description
 *
 * The version of the OpenAPI document: 0.21.2
 *
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

/// JobsResponse : Jobs collection response
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct JobsResponse {
    /// A list of jobs
    #[serde(rename = "jobs", skip_serializing_if = "Option::is_none")]
    pub jobs: Option<Vec<models::JobResponse>>,
    /// Total number of jobs for the user
    #[serde(rename = "count", skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    /// Offset at which paginated results are returned
    #[serde(rename = "offset")]
    pub offset: i32,
    /// Maximum number of results returned in the paginated response
    #[serde(rename = "limit")]
    pub limit: i32,
}

impl JobsResponse {
    /// Jobs collection response
    pub fn new(offset: i32, limit: i32) -> JobsResponse {
        JobsResponse {
            jobs: None,
            count: None,
            offset,
            limit,
        }
    }
}
