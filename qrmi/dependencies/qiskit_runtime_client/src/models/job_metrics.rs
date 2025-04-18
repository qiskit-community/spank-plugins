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

/// JobMetrics : Various metrics about the execution of a job
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct JobMetrics {
    #[serde(rename = "timestamps", skip_serializing_if = "Option::is_none")]
    pub timestamps: Option<Box<models::JobMetricsTimestamps>>,
    #[serde(rename = "bss", skip_serializing_if = "Option::is_none")]
    pub bss: Option<Box<models::JobMetricsBss>>,
    #[serde(rename = "usage", skip_serializing_if = "Option::is_none")]
    pub usage: Option<Box<models::JobMetricsUsage>>,
    /// Number of executions during job
    #[serde(rename = "executions", skip_serializing_if = "Option::is_none")]
    pub executions: Option<i32>,
    /// Number of circuits executed on quantum backend
    #[serde(rename = "num_circuits", skip_serializing_if = "Option::is_none")]
    pub num_circuits: Option<i32>,
    /// Number of qubits on quantum backend
    #[serde(rename = "num_qubits", skip_serializing_if = "Option::is_none")]
    pub num_qubits: Option<Vec<i32>>,
    /// An array of circuit depths
    #[serde(rename = "circuit_depths", skip_serializing_if = "Option::is_none")]
    pub circuit_depths: Option<Vec<i32>>,
    /// Qiskit version used during execution of the job
    #[serde(rename = "qiskit_version", skip_serializing_if = "Option::is_none")]
    pub qiskit_version: Option<String>,
    /// UTC timestamp for when the job will start
    #[serde(
        rename = "estimated_start_time",
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_start_time: Option<String>,
    /// UTC timestamp for when the job will complete
    #[serde(
        rename = "estimated_completion_time",
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_completion_time: Option<String>,
    /// Current position of job in queue (IBM Quantum channel users only)
    #[serde(rename = "position_in_queue", skip_serializing_if = "Option::is_none")]
    pub position_in_queue: Option<i32>,
    /// Current position of job in provider (IBM Quantum channel users only)
    #[serde(
        rename = "position_in_provider",
        skip_serializing_if = "Option::is_none"
    )]
    pub position_in_provider: Option<i32>,
}

impl JobMetrics {
    /// Various metrics about the execution of a job
    pub fn new() -> JobMetrics {
        JobMetrics {
            timestamps: None,
            bss: None,
            usage: None,
            executions: None,
            num_circuits: None,
            num_qubits: None,
            circuit_depths: None,
            qiskit_version: None,
            estimated_start_time: None,
            estimated_completion_time: None,
            position_in_queue: None,
            position_in_provider: None,
        }
    }
}
