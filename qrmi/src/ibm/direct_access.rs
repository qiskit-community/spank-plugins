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

use crate::models::{Payload, Target, TaskResult, TaskStatus};
use crate::QuantumResource;
use anyhow::{bail, Result};
use direct_access_api::utils::s3::S3Client;
use direct_access_api::{
    models::Backend, models::BackendStatus, models::Job, models::JobStatus, models::LogLevel,
    models::ProgramId, AuthMethod, Client, ClientBuilder,
};
use log::info;
use retry_policies::policies::ExponentialBackoff;
use retry_policies::Jitter;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

// c binding
use crate::consts::{QRMI_ERROR, QRMI_SUCCESS};
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_int};

use async_trait::async_trait;

/// QRMI implementation for IBM Qiskit Runtime Direct Access
pub struct IBMDirectAccess {
    pub(crate) api_client: Client,
    pub(crate) backend_name: String,
}

const DEFAULT_ENDPOINT: &str = "http://localhost:8080";

impl IBMDirectAccess {
    /// Constructs a QRMI to access IBM Qiskit Runtime Direct Access Service
    ///
    /// # Environment variables
    ///
    /// * `QRMI_IBM_DA_ENDPOINT`: IBM Qiskit Runtime Direct Access API endpoint URL
    /// * `QRMI_IBM_DA_AWS_ACCESS_KEY_ID`: AWS Access Key ID to access S3 bucket
    /// * `QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY`: AWS Secret Access Key to access S3 bucket
    /// * `QRMI_IBM_DA_S3_ENDPOINT`: S3 API endpoint URL
    /// * `QRMI_IBM_DA_S3_BUCKET`: S3 Bucket name
    /// * `QRMI_IBM_DA_S3_REGION`: S3 Region name
    /// * `QRMI_IBM_DA_IAM_ENDPOINT`: IBM Cloud IAM API endpoint URL
    /// * `QRMI_IBM_DA_IAM_APIKEY`: IBM Cloud API Key
    /// * `QRMI_IBM_DA_SERVICE_CRN`: Provisioned Direct Access Service instance
    /// * `QRMI_JOB_TIMEOUT_SECONDS`: Time (in seconds) after which job should time out and get cancelled.
    pub fn new(resource_id: &str) -> Self {
        // Check to see if the environment variables required to run this program are set.
        let daapi_endpoint = env::var(format!("{resource_id}_QRMI_IBM_DA_ENDPOINT"))
            .unwrap_or(DEFAULT_ENDPOINT.to_string());

        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
            .jitter(Jitter::Bounded)
            .base(2)
            .build_with_max_retries(5);

        let binding = ClientBuilder::new(daapi_endpoint);
        let mut builder = binding;
        builder
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(retry_policy);

        if let (
            Ok(aws_access_key_id),
            Ok(aws_secret_access_key),
            Ok(s3_endpoint),
            Ok(s3_bucket),
            Ok(s3_region),
        ) = (
            env::var(format!("{resource_id}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_S3_ENDPOINT")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_S3_BUCKET")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_S3_REGION")),
        ) {
            builder.with_s3bucket(
                &aws_access_key_id,
                &aws_secret_access_key,
                &s3_endpoint,
                &s3_bucket,
                &s3_region,
            );
        } else {
            info!("No S3 bucket configured.");
        }

        if let (Ok(apikey), Ok(service_crn), Ok(iam_endpoint_url)) = (
            env::var(format!("{resource_id}_QRMI_IBM_DA_IAM_APIKEY")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_SERVICE_CRN")),
            env::var(format!("{resource_id}_QRMI_IBM_DA_IAM_ENDPOINT")),
        ) {
            let auth_method = AuthMethod::IbmCloudIam {
                apikey,
                service_crn,
                iam_endpoint_url,
            };
            builder.with_auth(auth_method);
        } else {
            info!("No authentication configured.");
        }

        Self {
            api_client: builder.build().unwrap(),
            backend_name: resource_id.to_string(),
        }
    }
}

impl Default for IBMDirectAccess {
    fn default() -> Self {
        Self::new("")
    }
}
#[async_trait]
impl QuantumResource for IBMDirectAccess {
    async fn is_accessible(&mut self) -> bool {
        match self
            .api_client
            .get_backend::<Backend>(&self.backend_name)
            .await
        {
            Ok(val) => {
                if matches!(val.status, BackendStatus::Online) {
                    return true;
                }
                false
            }
            Err(_err) => {
                false
            }
        }
    }

    async fn acquire(&mut self) -> Result<String> {
        // Direct Access does not support session concept, so simply returns dummy ID for now.
        Ok(Uuid::new_v4().to_string())
    }

    async fn release(&mut self, _id: &str) -> Result<()> {
        // Direct Access does not support session concept, so simply ignores
        Ok(())
    }

    async fn task_start(&mut self, payload: Payload) -> Result<String> {
        let timeout = match env::var(format!("{0}_QRMI_JOB_TIMEOUT_SECONDS", self.backend_name)) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_JOB_TIMEOUT_SECONDS is not set: {}", &err));
            }
        };
        let timeout_secs = match timeout.parse::<u64>() {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("Failed to parse timeout value: {}", &err));
            }
        };

        if let Payload::QiskitPrimitive { input, program_id } = payload {
            let job: serde_json::Value = serde_json::from_str(input.as_str())?;
            if let Ok(program_id_enum) = ProgramId::from_str(&program_id) {
                match self
                    .api_client
                    .run_primitive(
                        &self.backend_name,
                        program_id_enum,
                        timeout_secs,
                        LogLevel::Debug,
                        &job,
                        None,
                    )
                    .await
                {
                    Ok(val) => Ok(val.job_id),
                    Err(err) => {
                        bail!(format!(
                            "An error occurred during starting a task: {}",
                            &err
                        ));
                    }
                }
            } else {
                bail!(format!("Unknown program ID is specified. {}", &program_id));
            }
        } else {
            bail!(format!("Payload type is not supported. {:?}", payload));
        }
    }

    async fn task_stop(&mut self, task_id: &str) -> Result<()> {
        let status = self.api_client.get_job_status(task_id).await?;
        if matches!(status, JobStatus::Running) {
            let _ = self.api_client.cancel_job(task_id, false).await;
        }
        self.api_client.delete_job(task_id).await?;
        Ok(())
    }

    async fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let status = self.api_client.get_job_status(task_id).await?;
        match status {
            JobStatus::Running => Ok(TaskStatus::Running),
            JobStatus::Completed => Ok(TaskStatus::Completed),
            JobStatus::Cancelled => Ok(TaskStatus::Cancelled),
            JobStatus::Failed => Ok(TaskStatus::Failed),
        }
    }

    async fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let s3_bucket = match env::var(format!("{0}_QRMI_IBM_DA_S3_BUCKET", self.backend_name)) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_IBM_DA_S3_BUCKET is not set: {}", &err));
            }
        };

        let s3_endpoint = match env::var(format!("{0}_QRMI_IBM_DA_S3_ENDPOINT", self.backend_name))
        {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_IBM_DA_S3_ENDPOINT is not set: {}", &err));
            }
        };

        let aws_access_key_id = match env::var(format!(
            "{0}_QRMI_IBM_DA_AWS_ACCESS_KEY_ID",
            self.backend_name
        )) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "QRMI_IBM_DA_AWS_ACCESS_KEY_ID is not set: {}",
                    &err
                ));
            }
        };

        let aws_secret_access_key = match env::var(format!(
            "{0}_QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY",
            self.backend_name
        )) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!(
                    "QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY is not set: {}",
                    &err
                ));
            }
        };

        let s3_region = match env::var(format!("{0}_QRMI_IBM_DA_S3_REGION", self.backend_name)) {
            Ok(val) => val,
            Err(err) => {
                bail!(format!("QRMI_IBM_DA_S3_REGION is not set: {}", &err));
            }
        };

        let s3_client = S3Client::new(
            s3_endpoint,
            aws_access_key_id,
            aws_secret_access_key,
            s3_region,
        );

        let job = self.api_client.get_job::<Job>(task_id).await?;
        if matches!(job.status, JobStatus::Failed) {
            let reason_code = job.reason_code.map_or("".to_string(), |v| v.to_string());
            let reason_message = job.reason_message.unwrap_or("".to_string());
            let reason_solution = job.reason_solution.unwrap_or("".to_string());
            bail!(
                format!(
                    "Unable to retrieve result for task {}. Task failed. code: {}, message: {}, solution: {}",
                    task_id, reason_code, reason_message, reason_solution
                )
            );
        }
        if matches!(job.status, JobStatus::Cancelled) {
            bail!(format!(
                "Unable to retrieve result for task {}. Task was cancelled.",
                task_id
            ));
        }
        if matches!(job.status, JobStatus::Running) {
            bail!(format!(
                "Unable to retrieve result for task {}. Task is running.",
                task_id
            ));
        }
        let s3_object_key = format!("results_{}.json", task_id);
        let object = s3_client.get_object(&s3_bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(TaskResult {
            value: retrieved_txt,
        })
    }

    async fn target(&mut self) -> Result<Target> {
        let mut resp = json!({});
        if let Ok(config) = self
            .api_client
            .get_backend_configuration::<serde_json::Value>(&self.backend_name)
            .await
        {
            resp["configuration"] = config;
        } else {
            resp["configuration"] = json!(null);
        }

        if let Ok(props) = self
            .api_client
            .get_backend_properties::<serde_json::Value>(&self.backend_name)
            .await
        {
            resp["properties"] = props;
        } else {
            resp["properties"] = json!(null);
        }

        Ok(Target {
            value: resp.to_string(),
        })
    }

    async fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}

// The following code is for C API binding.

/// @brief Returns a IBMDirectAccess QRMI handle.
///
/// Created IBMDirectAccess instance needs to be removed by qrmi_ibmda_free() call if
/// no longer needed.
///
/// # Safety
///
/// @param (resource_id) [in] A resource identifier, i.e. backend name
/// @return a IBMDirectAccess QRMI handle if succeeded, otherwise NULL. Must call qrmi_ibmda_free() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_new(resource_id: *const c_char) -> *mut IBMDirectAccess {
    ffi_helpers::null_pointer_check!(resource_id, std::ptr::null_mut());

    if let Ok(id_str) = CStr::from_ptr(resource_id).to_str() {
        let qrmi = Box::new(IBMDirectAccess::new(id_str));
        return Box::into_raw(qrmi);
    }
    std::ptr::null_mut()
}

/// @brief Returns true if device is accessible, otherwise false.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// * The memory pointed to by `outp` must have enough room to store boolean value.
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @param (outp) [out] accessible or not
/// @return QRMI_SUCCESS(0) if succeeded, otherwise QRMI_ERROR.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_is_accessible(
    qrmi: *mut IBMDirectAccess,
    outp: *mut bool,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(outp, QRMI_ERROR);

    let rt = tokio::runtime::Runtime::new().unwrap();
    *outp = rt.block_on(async {
        (*qrmi).is_accessible().await
    });
    QRMI_SUCCESS
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to qrmi_ibmda_new(). Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, returns < 0.
/// # Safety
///
/// * `ptr` must have been returned by a previous call to qrmi_ibmda_new().
///
/// @param (ptr) [in] A IBMDirectAccess QRMI handle
/// @return QRMI_SUCCESS(0) if succeeded, otherwise QRMI_ERROR.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_free(ptr: *mut IBMDirectAccess) -> c_int {
    if ptr.is_null() {
        return QRMI_ERROR;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    QRMI_SUCCESS
}

/// @brief Acquires quantum resource.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// * The memory pointed to by `outp` must have enough room to store boolean value.
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @return Acquisition token if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_acquire(qrmi: *mut IBMDirectAccess) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        (*qrmi).acquire().await
    });
    match result {
        Ok(token) => {
            if let Ok(token_cstr) = CString::new(token) {
                return token_cstr.into_raw();
            }
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }
    std::ptr::null()
}

/// @brief Releases quantum resource.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// * The memory pointed to by `acquisition_token` must contain a valid nul terminator at the
///   end of the string.
///
/// * The memory pointed to by `outp` must have enough room to store boolean value.
///
/// * `acquisition_token` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `acquisition_token` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `acquisition_token`
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @param (acquisition_token) [in] An acquisition token returned by qrmi_ibmda_acquire() call.
/// @return QRMI_SUCCESS if succeeded, otherwise QRMI_ERROR.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_release(
    qrmi: *mut IBMDirectAccess,
    acquisition_token: *const c_char,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(acquisition_token, QRMI_ERROR);

    if let Ok(token) = CStr::from_ptr(acquisition_token).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).release(token).await
        });
        match result {
            Ok(()) => {
                return QRMI_SUCCESS;
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    QRMI_SUCCESS
}

/// @brief Starts a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// * The memory pointed to by `program_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * The memory pointed to by `input` must contain a valid nul terminator at the
///   end of the string.
///
/// * `program_id` and `input` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `program_id`
///
/// * The nul terminator must be within `isize::MAX` from `input`
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @param (program_id) [in] Program ID (`sampler` or `estimator`)
/// @param (input) [in] primitive input
/// @return A task identifier if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_task_start(
    qrmi: *mut IBMDirectAccess,
    program_id: *const c_char,
    input: *const c_char,
) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    ffi_helpers::null_pointer_check!(program_id, std::ptr::null());
    ffi_helpers::null_pointer_check!(input, std::ptr::null());

    if let (Ok(program_id_str), Ok(input_str)) = (
        CStr::from_ptr(program_id).to_str(),
        CStr::from_ptr(input).to_str(),
    ) {
        let payload = Payload::QiskitPrimitive {
            input: input_str.to_string(),
            program_id: program_id_str.to_string(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_start(payload).await
        });
        match result {
            Ok(job_id) => {
                if let Ok(job_id_cstr) = CString::new(job_id) {
                    return job_id_cstr.into_raw();
                }
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    std::ptr::null()
}

/// @brief Stops a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `task_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `task_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @param (task_id) [in] A task ID, returned by a previous call to qrmi_ibmda_task_start()
/// @param (input) [in] Primitive input
/// @return QRMI_SUCCESS if succeeded, otherwise QRMI_ERROR.i
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_task_stop(
    qrmi: *mut IBMDirectAccess,
    task_id: *const c_char,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }

    ffi_helpers::null_pointer_check!(task_id, QRMI_ERROR);

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_stop(task_id_str).await
        });
        match result {
            Ok(()) => {
                return QRMI_SUCCESS;
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    QRMI_ERROR
}

/// @brief Returns the status of the specified task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * The memory pointed to by `outp` must have enough room to store `TaskStatus` value.
///
/// * `task_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `task_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @param (task_id) [in] A task identifier
/// @return QRMI_SUCCESS if succeeded, otherwise QRMI_ERROR.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_task_status(
    qrmi: *mut IBMDirectAccess,
    task_id: *const c_char,
    outp: *mut TaskStatus,
) -> c_int {
    if qrmi.is_null() {
        return QRMI_ERROR;
    }

    ffi_helpers::null_pointer_check!(task_id, QRMI_ERROR);
    ffi_helpers::null_pointer_check!(outp, QRMI_ERROR);

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_status(task_id_str).await
        });
        match result {
            Ok(v) => {
                *outp = v;
                return QRMI_SUCCESS;
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    QRMI_ERROR
}

/// @brief Returns the result of a task.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// * The memory pointed to by `task_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `task_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `task_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `task_id`
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @param (task_id) [in] A task identifier
/// @return Task result if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_task_result(
    qrmi: *mut IBMDirectAccess,
    task_id: *const c_char,
) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    ffi_helpers::null_pointer_check!(task_id, std::ptr::null());

    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            (*qrmi).task_result(task_id_str).await
        });
        match result {
            Ok(v) => {
                if let Ok(result_cstr) = CString::new(v.value) {
                    return result_cstr.into_raw();
                }
            }
            Err(err) => {
                eprintln!("{:?}", err);
            }
        }
    }
    std::ptr::null()
}

/// @brief Returns a Target for the specified device. Vendor specific serialized data. This might contain the constraints(instructions, properties and timing information etc.) of a particular device to allow compilers to compile an input circuit to something that works and is optimized for a device. In IBM implementation, it contains JSON representations of [BackendConfiguration](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_configuration_schema.json) and [BackendProperties](https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/backend_properties_schema.json) so that we are able to create a Target object by calling `qiskit_ibm_runtime.utils.backend_converter.convert_to_target` or uquivalent functions.
///
/// # Safety
///
/// * `qrmi` must have been returned by a previous call to qrmi_ibmda_new().
///
/// @param (qrmi) [in] A IBMDirectAccess QRMI handle
/// @return A serialized target data if succeeded, otherwise NULL. Must call qrmi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmda_target(qrmi: *mut IBMDirectAccess) -> *const c_char {
    if qrmi.is_null() {
        return std::ptr::null();
    }

    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(async {
        (*qrmi).target().await
    });
    match result {
        Ok(v) => {
            if let Ok(target_cstr) = CString::new(v.value) {
                return target_cstr.into_raw();
            }
        }
        Err(err) => {
            eprintln!("{:?}", err);
        }
    }
    std::ptr::null()
}
