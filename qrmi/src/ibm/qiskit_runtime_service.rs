//
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
    models::Backend, models::BackendStatus, models::JobStatus, models::LogLevel, models::ProgramId,
    AuthMethod, Client, ClientBuilder,
};
use retry_policies::policies::ExponentialBackoff;
use retry_policies::Jitter;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use std::time::Duration;

// python binding
use pyo3::prelude::*;

// c binding
use crate::consts::{QRMI_SUCCESS, QRMI_ERROR};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};

/// QRMI implementation for IBM Qiskit Runtime Service (QRS)
#[pyclass]
pub struct IBMQiskitRuntimeService {
    pub(crate) api_client: Client,
    pub(crate) s3_client: S3Client,
    pub(crate) backend_name: String,
    pub(crate) s3_bucket: String,
    pub(crate) timeout_secs: u64,
    pub(crate) session_mode: String,
    pub(crate) session_ttl: u64,
}

#[pymethods]
impl IBMQiskitRuntimeService {
    /// Constructs a QRMI to access IBM Qiskit Runtime Service.
    ///
    /// Environment variables (for QRS) include:
    /// * `QRMI_RESOURCE_ID`         - IBM Quantum backend name
    /// * `QRMI_IBM_QRS_ENDPOINT`    - Qiskit Runtime Service API endpoint URL
    /// * `QRMI_IBM_QRS_AWS_ACCESS_KEY_ID`
    /// * `QRMI_IBM_QRS_AWS_SECRET_ACCESS_KEY`
    /// * `QRMI_IBM_QRS_S3_ENDPOINT`
    /// * `QRMI_IBM_QRS_S3_BUCKET`
    /// * `QRMI_IBM_QRS_S3_REGION`
    /// * `QRMI_IBM_QRS_IAM_ENDPOINT`
    /// * `QRMI_IBM_QRS_IAM_APIKEY`
    /// * `QRMI_IBM_QRS_SERVICE_CRN`
    /// * `QRMI_IBM_QRS_TIMEOUT_SECONDS`
    /// * `QRMI_IBM_QRS_SESSION_MODE`
    /// * `QRMI_IBM_QRS_SESSION_TTL`
    #[new]
    pub fn new() -> Self {
        // Retrieve QRS-specific environment variables.
        let backend_name = env::var("QRMI_RESOURCE_ID").expect("QRMI_RESOURCE_ID");
        let qrs_endpoint = env::var("QRMI_IBM_QRS_ENDPOINT").expect("QRMI_IBM_QRS_ENDPOINT");
        let aws_access_key_id =
            env::var("QRMI_IBM_QRS_AWS_ACCESS_KEY_ID").expect("QRMI_IBM_QRS_AWS_ACCESS_KEY_ID");
        let aws_secret_access_key = env::var("QRMI_IBM_QRS_AWS_SECRET_ACCESS_KEY")
            .expect("QRMI_IBM_QRS_AWS_SECRET_ACCESS_KEY");
        let s3_endpoint = env::var("QRMI_IBM_QRS_S3_ENDPOINT").expect("QRMI_IBM_QRS_S3_ENDPOINT");
        let s3_bucket = env::var("QRMI_IBM_QRS_S3_BUCKET").expect("QRMI_IBM_QRS_S3_BUCKET");
        let s3_region = env::var("QRMI_IBM_QRS_S3_REGION").expect("QRMI_IBM_QRS_S3_REGION");

        let iam_endpoint_url =
            env::var("QRMI_IBM_QRS_IAM_ENDPOINT").expect("QRMI_IBM_QRS_IAM_ENDPOINT");
        let apikey = env::var("QRMI_IBM_QRS_IAM_APIKEY").expect("QRMI_IBM_QRS_IAM_APIKEY");
        let service_crn = env::var("QRMI_IBM_QRS_SERVICE_CRN").expect("QRMI_IBM_QRS_SERVICE_CRN");

        let timeout = env::var("QRMI_IBM_QRS_TIMEOUT_SECONDS").expect("QRMI_IBM_QRS_TIMEOUT_SECONDS");
        let timeout_secs = timeout.parse::<u64>().expect("QRMI_IBM_QRS_TIMEOUT_SECONDS");

        // Read session configuration from environment variables.
        // Defaults: mode = "dedicated", TTL = 28800 seconds.
        let session_mode = env::var("QRMI_IBM_QRS_SESSION_MODE")
            .unwrap_or_else(|_| "dedicated".to_string());
        let session_ttl_str = env::var("QRMI_IBM_QRS_SESSION_TTL")
            .unwrap_or_else(|_| "28800".to_string());
        let session_ttl = session_ttl_str.parse::<u64>().unwrap_or(28800);


        let retry_policy = ExponentialBackoff::builder()
            .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
            .jitter(Jitter::Bounded)
            .base(2)
            .build_with_max_retries(5);

        let auth_method = AuthMethod::IbmCloudIam {
            apikey,
            service_crn,
            iam_endpoint_url,
        };

        let api_client = ClientBuilder::new(qrs_endpoint)
            .with_timeout(Duration::from_secs(60))
            .with_retry_policy(retry_policy)
            .with_s3bucket(
                &aws_access_key_id,
                &aws_secret_access_key,
                &s3_endpoint,
                &s3_bucket,
                &s3_region,
            )
            .with_auth(auth_method)
            .build()
            .unwrap();

        let s3_client = S3Client::new(
            s3_endpoint,
            aws_access_key_id,
            aws_secret_access_key,
            s3_region,
        );

        Self {
            api_client,
            s3_client,
            backend_name,
            s3_bucket,
            timeout_secs,
            session_mode,
            session_ttl,
        }
    }

    #[pyo3(name = "is_accessible")]
    fn pyfunc_is_accessible(&mut self, id: &str) -> PyResult<bool> {
        Ok(self.is_accessible(id))
    }

    #[pyo3(name = "acquire")]
    fn pyfunc_acquire(&mut self, id: &str) -> PyResult<String> {
        match self.acquire(id) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into()),
        }
    }

    #[pyo3(name = "release")]
    fn pyfunc_release(&mut self, id: &str) -> PyResult<()> {
        match self.release(id) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[pyo3(name = "task_start")]
    fn pyfunc_task_start(&mut self, payload: Payload) -> PyResult<String> {
        match self.task_start(payload) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into()),
        }
    }

    #[pyo3(name = "task_stop")]
    fn pyfunc_task_stop(&mut self, task_id: &str) -> PyResult<()> {
        match self.task_stop(task_id) {
            Ok(()) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    #[pyo3(name = "task_status")]
    fn pyfunc_task_status(&mut self, task_id: &str) -> PyResult<TaskStatus> {
        match self.task_status(task_id) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into()),
        }
    }

    #[pyo3(name = "task_result")]
    fn pyfunc_task_result(&mut self, task_id: &str) -> PyResult<TaskResult> {
        match self.task_result(task_id) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into()),
        }
    }

    #[pyo3(name = "target")]
    fn pyfunc_target(&mut self, id: &str) -> PyResult<Target> {
        match self.target(id) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.into()),
        }
    }

    #[pyo3(name = "metadata")]
    fn pyfunc_metadata(&mut self) -> PyResult<HashMap<String, String>> {
        let mut metadata = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        Ok(metadata)
    }
}

impl Default for IBMQiskitRuntimeService {
    fn default() -> Self {
        Self::new()
    }
}

impl IBMQiskitRuntimeService {
    /// Wrapper of async call for QRS is_accessible() function.
    #[tokio::main]
    async fn _is_accessible(&mut self, id: &str) -> bool {
        match self.api_client.get_backend::<Backend>(id).await {
            Ok(val) => matches!(val.status, BackendStatus::Online),
            Err(_) => false,
        }
    }

    /// Wrapper of async call for QRS task_start() function.
    #[tokio::main]
    async fn _task_start(&mut self, payload: Payload) -> Result<String> {
        if let Payload::QiskitPrimitive { input, program_id } = payload {
            let job: serde_json::Value = serde_json::from_str(&input)?;
            if let Ok(program_id_enum) = ProgramId::from_str(&program_id) {
                match self
                    .api_client
                    .run_primitive(
                        &self.backend_name,
                        program_id_enum,
                        self.timeout_secs,
                        LogLevel::Debug,
                        &job,
                        None,
                    )
                    .await
                {
                    Ok(val) => Ok(val.job_id),
                    Err(err) => bail!("Error starting task: {}", err),
                }
            } else {
                bail!("Unknown program ID: {}", program_id)
            }
        } else {
            bail!("Unsupported payload type: {:?}", payload)
        }
    }

    /// Wrapper of async call for QRS task_stop() function.
    #[tokio::main]
    async fn _task_stop(&mut self, task_id: &str) -> Result<()> {
        let status = self.api_client.get_job_status(task_id).await?;
        if matches!(status, JobStatus::Running) {
            let _ = self.api_client.cancel_job(task_id, false).await;
        }
        self.api_client.delete_job(task_id).await?;
        Ok(())
    }

    /// Wrapper of async call for QRS task_status() function.
    #[tokio::main]
    async fn _task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        let status = self.api_client.get_job_status(task_id).await?;
        Ok(match status {
            JobStatus::Running => TaskStatus::Running,
            JobStatus::Completed => TaskStatus::Completed,
            JobStatus::Cancelled => TaskStatus::Cancelled,
            JobStatus::Failed => TaskStatus::Failed,
        })
    }

    /// Wrapper of async call for QRS task_result() function.
    #[tokio::main]
    async fn _task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        let s3_object_key = format!("results_{}.json", task_id);
        let object = self.s3_client.get_object(&self.s3_bucket, &s3_object_key).await?;
        let retrieved_txt = String::from_utf8(object)?;
        Ok(TaskResult { value: retrieved_txt })
    }

    /// Wrapper of async call for QRS target() function.
    #[tokio::main]
    async fn _target(&mut self, id: &str) -> Result<Target> {
        let mut resp = json!({});
        if let Ok(config) = self.api_client.get_backend_configuration::<serde_json::Value>(id).await {
            resp["configuration"] = config;
        } else {
            resp["configuration"] = json!(null);
        }
        if let Ok(props) = self.api_client.get_backend_properties::<serde_json::Value>(id).await {
            resp["properties"] = props;
        } else {
            resp["properties"] = json!(null);
        }
        Ok(Target { value: resp.to_string() })
    }

    /// Acquires a new session by calling the get_session function.
    #[tokio::main]
    async fn _acquire_session(&mut self) -> Result<String> {
        self.api_client.get_session(&self.session_mode, self.session_ttl).await
    }
    

    /// Releases an existing session by issuing a DELETE to the session endpoint.
    #[tokio::main]
    async fn _release_session(&mut self, session_id: &str) -> Result<()> {
        let url = format!("{}/v1/sessions/{}", self.api_client.base_url, session_id);
        let resp = self.api_client.client.delete(url).send().await?;
        if resp.status().is_success() {
            Ok(())
        } else {
            bail!("Failed to release session: {}", resp.status())
        }
    }
}

impl QuantumResource for IBMQiskitRuntimeService {
    fn is_accessible(&mut self, id: &str) -> bool {
        self._is_accessible(id)
    }

    fn acquire(&mut self, _id: &str) -> Result<String> {
        self._acquire_session()
    }

    fn release(&mut self, id: &str) -> Result<()> {
        self._release_session(id)
    }

    fn task_start(&mut self, payload: Payload) -> Result<String> {
        self._task_start(payload)
    }

    fn task_stop(&mut self, task_id: &str) -> Result<()> {
        self._task_stop(task_id)
    }

    fn task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        self._task_status(task_id)
    }

    fn task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        self._task_result(task_id)
    }

    fn target(&mut self, id: &str) -> Result<Target> {
        self._target(id)
    }

    fn metadata(&mut self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("backend_name".to_string(), self.backend_name.clone());
        metadata
    }
}

// C API bindings for QRS

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_new() -> *mut IBMQiskitRuntimeService {
    let qrs = Box::new(IBMQiskitRuntimeService::new());
    Box::into_raw(qrs)
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_is_accessible(
    qrs: *mut IBMQiskitRuntimeService,
    id: *const c_char,
    outp: *mut bool,
) -> c_int {
    if qrs.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(id, QRMI_ERROR);
    ffi_helpers::null_pointer_check!(outp, QRMI_ERROR);
    if let Ok(id_str) = CStr::from_ptr(id).to_str() {
        *outp = (*qrs).is_accessible(id_str);
        return QRMI_SUCCESS;
    }
    QRMI_ERROR
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_free(ptr: *mut IBMQiskitRuntimeService) -> c_int {
    if ptr.is_null() {
        return QRMI_ERROR;
    }
    let _ = Box::from_raw(ptr);
    QRMI_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_acquire(
    qrs: *mut IBMQiskitRuntimeService,
    id: *const c_char,
) -> *const c_char {
    if qrs.is_null() {
        return std::ptr::null();
    }
    ffi_helpers::null_pointer_check!(id, std::ptr::null());
    // In QRS, the provided id is ignored; we acquire a session.
    match (*qrs).acquire("") {
        Ok(token) => {
            if let Ok(token_cstr) = CString::new(token) {
                return token_cstr.into_raw();
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
        }
    }
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_release(
    qrs: *mut IBMQiskitRuntimeService,
    id: *const c_char,
) -> c_int {
    if qrs.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(id, QRMI_ERROR);
    if let Ok(token) = CStr::from_ptr(id).to_str() {
        match (*qrs).release(token) {
            Ok(()) => return QRMI_SUCCESS,
            Err(e) => eprintln!("{:?}", e),
        }
    }
    QRMI_ERROR
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_task_start(
    qrs: *mut IBMQiskitRuntimeService,
    program_id: *const c_char,
    input: *const c_char,
) -> *const c_char {
    if qrs.is_null() {
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
        match (*qrs).task_start(payload) {
            Ok(job_id) => {
                if let Ok(job_id_cstr) = CString::new(job_id) {
                    return job_id_cstr.into_raw();
                }
            }
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_task_stop(
    qrs: *mut IBMQiskitRuntimeService,
    task_id: *const c_char,
) -> c_int {
    if qrs.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(task_id, QRMI_ERROR);
    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        match (*qrs).task_stop(task_id_str) {
            Ok(()) => return QRMI_SUCCESS,
            Err(e) => eprintln!("{:?}", e),
        }
    }
    QRMI_ERROR
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_task_status(
    qrs: *mut IBMQiskitRuntimeService,
    task_id: *const c_char,
    outp: *mut TaskStatus,
) -> c_int {
    if qrs.is_null() {
        return QRMI_ERROR;
    }
    ffi_helpers::null_pointer_check!(task_id, QRMI_ERROR);
    ffi_helpers::null_pointer_check!(outp, QRMI_ERROR);
    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        match (*qrs).task_status(task_id_str) {
            Ok(status) => {
                *outp = TaskStatus::from(status);
                return QRMI_SUCCESS;
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    QRMI_ERROR
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_task_result(
    qrs: *mut IBMQiskitRuntimeService,
    task_id: *const c_char,
) -> *const c_char {
    if qrs.is_null() {
        return std::ptr::null();
    }
    ffi_helpers::null_pointer_check!(task_id, std::ptr::null());
    if let Ok(task_id_str) = CStr::from_ptr(task_id).to_str() {
        match (*qrs).task_result(task_id_str) {
            Ok(result) => {
                if let Ok(result_cstr) = CString::new(result.value) {
                    return result_cstr.into_raw();
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    std::ptr::null()
}

#[no_mangle]
pub unsafe extern "C" fn qrmi_ibmqrs_target(
    qrs: *mut IBMQiskitRuntimeService,
    id: *const c_char,
) -> *const c_char {
    if qrs.is_null() {
        return std::ptr::null();
    }
    ffi_helpers::null_pointer_check!(id, std::ptr::null());
    if let Ok(id_str) = CStr::from_ptr(id).to_str() {
        match (*qrs).target(id_str) {
            Ok(target) => {
                if let Ok(target_cstr) = CString::new(target.value) {
                    return target_cstr.into_raw();
                }
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    std::ptr::null()
}
