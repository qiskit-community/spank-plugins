// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This code is licensed under the Apache License, Version 2.0. You may
// obtain a copy of this license in the LICENSE.txt file in the root directory
// of this source tree or at http://www.apache.org/licenses/LICENSE-2.0.
//
// Any modifications or derivative works of this code must retain this
// copyright notice, and modified files need to carry a notice indicating
// that they have been altered from the originals.

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
/// QRMI resource types
pub enum ResourceType {
    /// IBM Direct Access
    IBMDirectAccess,
    /// Qiskit Runtime Service
    QiskitRuntimeService,
    /// Pasqal Cloud
    PasqalCloud,
}
impl<'de> serde::Deserialize<'de> for ResourceType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "direct-access" => Ok(ResourceType::IBMDirectAccess),
            "qiskit-runtime-service" => Ok(ResourceType::QiskitRuntimeService),
            "pasqal-cloud" => Ok(ResourceType::PasqalCloud),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["direct-access", "qiskit-runtime-service", "pasqal-cloud"],
            )),
        }
    }
}
impl ResourceType {
    pub(crate) fn as_str(&self) -> &str {
        match self {
            ResourceType::IBMDirectAccess => "direct-access",
            ResourceType::QiskitRuntimeService => "qiskit-runtime-service",
            ResourceType::PasqalCloud => "pasqal-cloud",
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
/// A QRMI resource
pub struct QRMIResource {
    /// resource name
    pub name: String,

    /// resource type
    pub r#type: ResourceType,

    /// environment variables
    pub environment: HashMap<String, String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
/// A list of [`QRMIResource`] specified to this Slurm job.
pub struct QRMIResources {
    pub resources: Vec<QRMIResource>,
}
