// This code is part of Qiskit.
//
// (C) Copyright IBM 2025
//
// This program and the accompanying materials are made available under the
// terms of the GNU General Public License version 3, as published by the
// Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <[https://www.gnu.org/licenses/gpl-3.0.txt]
//

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
