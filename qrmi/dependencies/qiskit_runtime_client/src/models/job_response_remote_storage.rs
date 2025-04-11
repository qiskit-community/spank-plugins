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
pub struct JobResponseRemoteStorage {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<Type>,
    /// Region, Cross-Region, or Single Data Center as defined by IBM Cloud Object Storage (https://cloud.ibm.com/docs/cloud-object-storage?topic=cloud-object-storage-endpoints)
    #[serde(rename = "region")]
    pub region: String,
    #[serde(rename = "region_type")]
    pub region_type: RegionType,
    /// Fully specified <a href=\"https://cloud.ibm.com/docs/account?topic=account-crn\">CRN</a> for the target Cloud Object Storage bucket
    #[serde(rename = "bucket_crn")]
    pub bucket_crn: String,
    /// Name/ID of the object in the IBM Cloud Object Storage bucket. May *not* be specified in the default remote storage option.
    #[serde(rename = "object_name")]
    pub object_name: String,
    #[serde(rename = "job_params")]
    pub job_params: Box<models::RemoteStorageJobParams>,
    #[serde(rename = "results")]
    pub results: Box<models::RemoteStorageResults>,
    #[serde(rename = "logs", skip_serializing_if = "Option::is_none")]
    pub logs: Option<Box<models::RemoteStorageLogs>>,
    #[serde(
        rename = "transpiled_circuits",
        skip_serializing_if = "Option::is_none"
    )]
    pub transpiled_circuits: Option<Box<models::RemoteStorageTranspiledCircuits>>,
}

impl JobResponseRemoteStorage {
    pub fn new(
        region: String,
        region_type: RegionType,
        bucket_crn: String,
        object_name: String,
        job_params: models::RemoteStorageJobParams,
        results: models::RemoteStorageResults,
    ) -> JobResponseRemoteStorage {
        JobResponseRemoteStorage {
            r#type: None,
            region,
            region_type,
            bucket_crn,
            object_name,
            job_params: Box::new(job_params),
            results: Box::new(results),
            logs: None,
            transpiled_circuits: None,
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "ibmcloud_cos")]
    IbmcloudCos,
}

impl Default for Type {
    fn default() -> Type {
        Self::IbmcloudCos
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum RegionType {
    #[serde(rename = "regional")]
    Regional,
    #[serde(rename = "cross-region")]
    CrossRegion,
    #[serde(rename = "single-site")]
    SingleSite,
}

impl Default for RegionType {
    fn default() -> RegionType {
        Self::Regional
    }
}
