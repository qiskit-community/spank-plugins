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

/// Usage : usage metrics
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Usage {
    /// Number of seconds of Qiskit Runtime usage including quantum compute and near-time classical pre- and post-processing
    #[serde(rename = "seconds")]
    pub seconds: f64,
}

impl Usage {
    /// usage metrics
    pub fn new(seconds: f64) -> Usage {
        Usage { seconds }
    }
}
