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
pub struct VersionResponse {
    #[serde(rename = "versions")]
    pub versions: Vec<String>,
}

impl VersionResponse {
    pub fn new(versions: Vec<String>) -> VersionResponse {
        VersionResponse { versions }
    }
}
