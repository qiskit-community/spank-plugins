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

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateJobRequestOneOf1 {
    /// ID of the program to be executed
    #[serde(rename = "program_id")]
    pub program_id: String,
    /// Name that identifies the backend on which to run the program.
    #[serde(rename = "backend")]
    pub backend: String,
    /// Name and tag of the image to use when running a program (IBM Quantum channel users only). Should follow the pattern \"name:tag\".
    #[serde(rename = "runtime", skip_serializing_if = "Option::is_none")]
    pub runtime: Option<String>,
    /// List of job or program tags
    #[serde(rename = "tags", skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Logging level of the program
    #[serde(rename = "log_level", skip_serializing_if = "Option::is_none")]
    pub log_level: Option<LogLevel>,
    /// Cost of the job as the estimated time it should take to complete (in seconds). Should not exceed the cost of the program
    #[serde(rename = "cost", skip_serializing_if = "Option::is_none")]
    pub cost: Option<u64>,
    /// Identifier of the session that the job is a part of
    #[serde(rename = "session_id", skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(rename = "remote_storage")]
    pub remote_storage: Box<models::JobResponseRemoteStorage>,
}

impl CreateJobRequestOneOf1 {
    pub fn new(
        program_id: String,
        backend: String,
        remote_storage: models::JobResponseRemoteStorage,
    ) -> CreateJobRequestOneOf1 {
        CreateJobRequestOneOf1 {
            program_id,
            backend,
            runtime: None,
            tags: None,
            log_level: None,
            cost: None,
            session_id: None,
            remote_storage: Box::new(remote_storage),
        }
    }
}
/// Logging level of the program
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum LogLevel {
    #[serde(rename = "critical")]
    Critical,
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "warning")]
    Warning,
    #[serde(rename = "info")]
    Info,
    #[serde(rename = "debug")]
    Debug,
}

impl Default for LogLevel {
    fn default() -> LogLevel {
        Self::Critical
    }
}
