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

use anyhow::Result;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::{c_char, c_float, c_int, c_uint, c_ulong};
use std::sync::Once;
use std::time::Duration;

use retry_policies::{policies::ExponentialBackoff, Jitter};
use serde::de::DeserializeOwned;

use direct_access_api::AuthMethod;

use crate::consts::{DAAPI_ERROR, DAAPI_SUCCESS};

static INIT: Once = Once::new();

/// @brief Direct Access API client handle
pub struct Client {
    #[allow(dead_code)]
    internal: direct_access_api::Client,
}

/// @brief A builder to create Client
pub struct ClientBuilder {
    #[allow(dead_code)]
    internal: direct_access_api::ClientBuilder,
}

/// @brief A Primitive job handle
pub struct PrimitiveJob {
    #[allow(dead_code)]
    internal: direct_access_api::PrimitiveJob,
}

/// Status of the backend
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackendStatus {
    /// online (you can send jobs)
    ONLINE = 1,
    /// offline (you cannot send jobs)
    OFFLINE = 2,
    /// paused (you cannot send jobs)
    PAUSED = 3,
}
impl<'de> serde::Deserialize<'de> for BackendStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.as_str() {
            "online" => Ok(BackendStatus::ONLINE),
            "offline" => Ok(BackendStatus::OFFLINE),
            "paused" => Ok(BackendStatus::PAUSED),
            _ => Err(serde::de::Error::unknown_variant(
                &s,
                &["online", "offline", "paused"],
            )),
        }
    }
}

// IR of Backend
#[derive(Debug, Clone, serde::Deserialize)]
struct BackendIntermediate {
    /// Name of the backend
    pub name: String,

    /// Enum: "online" "offline" "paused"
    pub status: BackendStatus,
}

// IR of BackendList
#[derive(Debug, Clone, serde::Deserialize)]
struct BackendListIntermediate {
    pub backends: Vec<BackendIntermediate>,
}

/// @brief Details of quantum backend
#[repr(C)]
#[derive(Debug)]
pub struct Backend {
    /// backend name
    name: *mut c_char,
    /// backend status
    status: BackendStatus,
}

/// @brief A list of available quantum backends
#[repr(C)]
#[derive(Debug)]
pub struct BackendList {
    /// Ptr to the first Backend in the list
    backends: *mut Backend,
    /// Number of Backends included in the list
    length: usize,
}

/// @brief Status of submitted job
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    /// job is running
    RUNNING = 1,
    /// job was completed
    COMPLETED = 2,
    /// job was failed
    FAILED = 3,
    /// job was cancelled
    CANCELLED = 4,
}
impl From<direct_access_api::models::JobStatus> for JobStatus {
    fn from(status: direct_access_api::models::JobStatus) -> Self {
        match status {
            direct_access_api::models::JobStatus::Running => JobStatus::RUNNING,
            direct_access_api::models::JobStatus::Completed => JobStatus::COMPLETED,
            direct_access_api::models::JobStatus::Failed => JobStatus::FAILED,
            direct_access_api::models::JobStatus::Cancelled => JobStatus::CANCELLED,
        }
    }
}

/// @brief Primitive types
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum ProgramId {
    /// Estimator
    ESTIMATOR = 1,
    /// Sampler
    SAMPLER = 2,
}
impl From<ProgramId> for direct_access_api::models::ProgramId {
    fn from(val: ProgramId) -> Self {
        match val {
            ProgramId::ESTIMATOR => direct_access_api::models::ProgramId::Estimator,
            ProgramId::SAMPLER => direct_access_api::models::ProgramId::Sampler,
        }
    }
}
impl From<direct_access_api::models::ProgramId> for ProgramId {
    fn from(status: direct_access_api::models::ProgramId) -> Self {
        match status {
            direct_access_api::models::ProgramId::Estimator => ProgramId::ESTIMATOR,
            direct_access_api::models::ProgramId::Sampler => ProgramId::SAMPLER,
        }
    }
}

/// @brief Metrics of job
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Metrics {
    /// Time when job was created.
    created_time: *mut c_char,
    /// Time when job reached a terminal status. NULL if job is running.
    end_time: *mut c_char,
    /// Execution time on quantum device in nanoseconds. 0 if job is running.
    quantum_nanoseconds: i64,
}

fn _to_metrics(job: direct_access_api::models::Job) -> Metrics {
    if let Ok(created_time) = CString::new(job.created_time) {
        let end_time = match job.end_time {
            Some(val) => {
                if let Ok(c_str) = CString::new(val) {
                    c_str.into_raw()
                } else {
                    std::ptr::null()
                }
            }
            None => std::ptr::null(),
        };
        let quantum_nanoseconds = job.usage.quantum_nanoseconds.unwrap_or(0);
        return Metrics {
            created_time: created_time.into_raw(),
            end_time: end_time as *mut c_char,
            quantum_nanoseconds,
        };
    }
    Metrics {
        created_time: std::ptr::null_mut::<c_char>(),
        end_time: std::ptr::null_mut::<c_char>(),
        quantum_nanoseconds: 0,
    }
}

/// @brief Metadata of submitted job
#[repr(C)]
#[derive(Debug)]
pub struct Job {
    /// Job ID
    id: *mut c_char,
    /// Job status
    status: JobStatus,
    /// Program Id
    program_id: ProgramId,
    /// Metrics
    metrics: Metrics,
}

/// @brief A list of jobs in Direct Access service
#[repr(C)]
#[derive(Debug)]
pub struct JobList {
    /// Ptr to the first Job in the list
    jobs: *mut Job,
    /// Number of jobs included in the list
    length: usize,
}

/// @brief Logging levels
#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    /// Logs that describe an unrecoverable application or system crash, or a catastrophic failure that requires immediate attention.
    CRITICAL = 1,
    /// Logs that highlight when the current flow of execution is stopped due to a failure.
    ERROR = 2,
    /// Logs that highlight an abnormal or unexpected event in the application flow, but do not otherwise cause the application execution to stop.
    WARNING = 3,
    /// Logs that track the general flow of the application. These logs should have long-term value.
    INFO = 4,
    /// Logs that are used for interactive investigation during development. These logs should primarily contain information useful for debugging and have no long-term value.
    DEBUG = 5,
}
impl From<LogLevel> for direct_access_api::models::LogLevel {
    fn from(val: LogLevel) -> Self {
        match val {
            LogLevel::CRITICAL => direct_access_api::models::LogLevel::Critical,
            LogLevel::ERROR => direct_access_api::models::LogLevel::Error,
            LogLevel::WARNING => direct_access_api::models::LogLevel::Warning,
            LogLevel::INFO => direct_access_api::models::LogLevel::Info,
            LogLevel::DEBUG => direct_access_api::models::LogLevel::Debug,
        }
    }
}

/// @brief Must call once before using the C API library to initialize static resources(logger etc.) in underlying layers. If called more than once, the second and subsequent calls are ignored.
/// @version 0.1.0
#[no_mangle]
pub extern "C" fn daapi_init() {
    INIT.call_once(|| {
        env_logger::init();
    });
}

/// @brief Returns a ClientBuilder handle.
///
/// Created ClientBuilder instance needs to be removed by daapi_free_builder() call if
/// no longer needed.
///
/// # Safety
///
/// * The memory pointed to by `endpoint_url` must contain a valid nul terminator at the
///   end of the string.
///
/// * `endpoint_url` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `endpoint_url` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `endpoint_url`
///
/// @param (endpoint_url) [in] Direct Access API endpoint URL
/// @return a ClientBuilder handle if succeeded, otherwise NULL. Must call daapi_free_builder() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_bldr_new(endpoint_url: *const c_char) -> *mut ClientBuilder {
    ffi_helpers::null_pointer_check!(endpoint_url, std::ptr::null_mut::<ClientBuilder>());
    if let Ok(base_url) = CStr::from_ptr(endpoint_url).to_str() {
        let builder = Box::new(ClientBuilder {
            internal: direct_access_api::ClientBuilder::new(base_url.to_string()),
        });
        return Box::into_raw(builder);
    }
    std::ptr::null_mut::<ClientBuilder>()
}

/// @brief Enables IBM Cloud IAM Bearer Token based authentication
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// * The memory pointed to by `apikey`/`crn`/`endpoint` must contain a valid nul terminator at the
///   end of the string.
///
/// * `apikey`/`crn`/`endpoint` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `apikey`/`crn`/`endpoint` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `apikey`/`crn`/`endpoint`
///
/// @param (builder) [in] ClientBuilder handle
/// @param (apikey) [in] IAM api key to generate token
/// @param (crn) [in] Service-CRN of provisioned instance. e.g. "crn:version:cname:ctype:service-name:location:scope:service-instance:resource-type:resource"
/// @param (endpoint) [in] IAM endpoint URL(e.g. https://iam.cloud.ibm.com)
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_bldr_enable_iam_auth(
    builder: &mut ClientBuilder,
    apikey: *const c_char,
    crn: *const c_char,
    endpoint: *const c_char,
) -> libc::c_int {
    ffi_helpers::null_pointer_check!(apikey, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(crn, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(endpoint, DAAPI_ERROR);
    if let (Ok(iam_apikey), Ok(service_crn), Ok(iam_endpoint_url)) = (
        CStr::from_ptr(apikey).to_str(),
        CStr::from_ptr(crn).to_str(),
        CStr::from_ptr(endpoint).to_str(),
    ) {
        builder.internal.with_auth(AuthMethod::IbmCloudIam {
            apikey: iam_apikey.to_string(),
            service_crn: service_crn.to_string(),
            iam_endpoint_url: iam_endpoint_url.to_string(),
        });
        return DAAPI_SUCCESS;
    }
    DAAPI_ERROR
}

/// @brief Enables IBM Cloud App ID access-token based authentication
/// @deprecated Use with_ibmcloud_iam_bearer_token_auth().
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// * The memory pointed to by `client_id`/`secret` must contain a valid nul terminator at the
///   end of the string.
///
/// * `client_id`/`secret` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `client_id`/`secret` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `client_id`/`secret`
///
/// @param (builder) [in] ClientBuilder handle
/// @param (client_id) [in] App ID username
/// @param (secret) [in] App ID password
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
#[cfg(feature = "ibmcloud_appid_auth")]
pub unsafe extern "C" fn daapi_bldr_enable_appid_auth(
    builder: &mut ClientBuilder,
    client_id: *const c_char,
    secret: *const c_char,
) -> libc::c_int {
    ffi_helpers::null_pointer_check!(client_id, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(secret, DAAPI_ERROR);
    if let (Ok(username), Ok(password)) = (
        CStr::from_ptr(client_id).to_str(),
        CStr::from_ptr(secret).to_str(),
    ) {
        builder.internal.with_auth(AuthMethod::IbmCloudAppId {
            username: username.to_string(),
            password: password.to_string(),
        });
        return DAAPI_SUCCESS;
    }
    DAAPI_ERROR
}

/// @brief Set a timeout for only the connect phase of a Client.
///
/// Default is no timeout.
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// @param (builder) [in] ClientBuilder handle
/// @param (seconds) [in] timeout in seconds
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub extern "C" fn daapi_bldr_set_connect_timeout(
    builder: &mut ClientBuilder,
    seconds: c_float,
) -> libc::c_int {
    builder
        .internal
        .with_connect_timeout(Duration::from_secs_f32(seconds));
    DAAPI_SUCCESS
}

/// @brief Enables a read timeout.
///
/// The timeout applies to each read operation, and resets after a
/// successful read. This is more appropriate for detecting stalled
/// connections when the size isn't known beforehand.
///
/// Default is no timeout.
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// @param (builder) [in] A ClientBuilder handle
/// @param (seconds) [in] timeout in seconds
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub extern "C" fn daapi_bldr_set_read_timeout(
    builder: &mut ClientBuilder,
    seconds: c_float,
) -> libc::c_int {
    builder
        .internal
        .with_read_timeout(Duration::from_secs_f32(seconds));
    DAAPI_SUCCESS
}

/// @brief Enables a total request timeout.
///
/// The timeout is applied from when the request starts connecting until the
/// response body has finished. Also considered a total deadline.
///
/// Default is no timeout.
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// @param (builder) [in] A ClientBuilder handle
/// @param (seconds) [in] timeout in seconds
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub extern "C" fn daapi_bldr_set_timeout(
    builder: &mut ClientBuilder,
    seconds: c_float,
) -> libc::c_int {
    builder
        .internal
        .with_timeout(Duration::from_secs_f32(seconds));
    DAAPI_SUCCESS
}

/// @brief Enables retry with a exponential backoff implementation
///
/// This implementation increases the backoff
/// period for each retry attempt using a randomization function that grows
/// exponentially.
///
/// Default is no retry.
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// @param (builder) [in] A ClientBuilder handle
/// @param (retries) [in] Maximum number of allowed retries attempts.
/// @param (base) [in] Base of the exponential
/// @param (min_seconds) [in] The initial retry interval.
/// @param (max_seconds) [in] Maximum retry interval.
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub extern "C" fn daapi_bldr_set_exponential_backoff_retry(
    builder: &mut ClientBuilder,
    retries: c_uint,
    base: c_uint,
    min_seconds: c_float,
    max_seconds: c_float,
) -> libc::c_int {
    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(
            Duration::from_secs_f32(min_seconds),
            Duration::from_secs_f32(max_seconds),
        )
        .jitter(Jitter::Bounded)
        .base(base)
        .build_with_max_retries(retries);
    let _ = builder.internal.with_retry_policy(retry_policy);
    DAAPI_SUCCESS
}

/// @brief Set S3 bucket connection parameters.
///
/// S3 bucket connection parameters are required to invoke daapi_cli_run_primitive().
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// * The memory pointed to by `aws_access_key_id`/`aws_secret_access_key`/`s3_endpoint`/`s3_bucket`/`s3_region` must contain a valid nul terminator at the
///   end of the string.
///
/// * `aws_access_key_id`/`aws_secret_access_key`/`s3_endpoint`/`s3_bucket`/`s3_region` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `aws_access_key_id`/`aws_secret_access_key`/`s3_endpoint`/`s3_bucket`/`s3_region` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `aws_access_key_id`/`aws_secret_access_key`/`s3_endpoint`/`s3_bucket`/`s3_region`
///
/// @param (builder) [in] A ClientBuilder handle
/// @param (aws_access_key_id) [in] AWS access key id
/// @param (aws_secret_access_key) [in] AWS secret access key
/// @param (s3_endpoint) [in] S3 endpoint URL
/// @param (s3_bucket) [in] S3 bucket name
/// @param (s3_region) [in] S3 region name
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_bldr_set_s3_bucket(
    builder: &mut ClientBuilder,
    aws_access_key_id: *const c_char,
    aws_secret_access_key: *const c_char,
    s3_endpoint: *const c_char,
    s3_bucket: *const c_char,
    s3_region: *const c_char,
) -> libc::c_int {
    ffi_helpers::null_pointer_check!(aws_access_key_id, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(aws_secret_access_key, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(s3_endpoint, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(s3_bucket, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(s3_region, DAAPI_ERROR);
    if let (Ok(access_key), Ok(secret), Ok(endpoint), Ok(bucket), Ok(region)) = (
        CStr::from_ptr(aws_access_key_id).to_str(),
        CStr::from_ptr(aws_secret_access_key).to_str(),
        CStr::from_ptr(s3_endpoint).to_str(),
        CStr::from_ptr(s3_bucket).to_str(),
        CStr::from_ptr(s3_region).to_str(),
    ) {
        builder.internal.with_s3bucket(
            access_key.to_string(),
            secret.to_string(),
            endpoint.to_string(),
            bucket.to_string(),
            region.to_string(),
        );
        return DAAPI_SUCCESS;
    }
    DAAPI_ERROR
}

/// @brief Sets a `IBM-API-Version` HTTP header value.
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// * The memory pointed to by `api_version` must contain a valid nul terminator at the
///   end of the string.
///
/// * `api_version` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `api_version` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `api_version`
///
/// @param (builder) [in] A ClientBuilder
/// @param (api_version) [in] version string in YYYY-MM-DD format.
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[cfg(feature = "api_version")]
#[no_mangle]
pub unsafe extern "C" fn daapi_bldr_set_api_version(
    builder: &mut ClientBuilder,
    api_version: *const c_char,
) -> libc::c_int {
    ffi_helpers::null_pointer_check!(api_version, DAAPI_ERROR);

    if let Ok(version) = CStr::from_ptr(api_version).to_str() {
        builder.internal.with_api_version(version);
        return DAAPI_SUCCESS;
    }
    DAAPI_ERROR
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_bldr_new() or related functions. Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_bldr_new().
///
/// @param (ptr) [in] a ClientBuilder
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_builder(ptr: *mut ClientBuilder) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    DAAPI_SUCCESS
}

/// @brief Returns a Client handle that uses the configuration set to the specified ClientBuilder.
///
/// # Safety
///
/// * `builder` must have been returned by a previous call to daapi_bldr_new().
///
/// @param (builder) [in] a ClientBuilder handle
/// @return a Client handle if succeeded, otherwise NULL. Must call daapi_free_client() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_new(builder: &mut ClientBuilder) -> *mut Client {
    if let Ok(internal) = builder.internal.build() {
        let client = Box::new(Client { internal });
        return Box::into_raw(client);
    }
    std::ptr::null_mut::<Client>()
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_bldr_client_new() or related functions. Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_cli_new().
///
/// @param (ptr) [in] a Client
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_client(ptr: *mut Client) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _get_backend_properties(
    client: *mut Client,
    backend: &str,
) -> Result<serde_json::Value> {
    (*client)
        .internal
        .get_backend_properties::<serde_json::Value>(backend)
        .await
}
/// @brief Returns the properties of the specified backend
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `backend` must contain a valid nul terminator at the
///   end of the string.
///
/// * `backend` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `backend` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `backend`
///
/// @param (client) [in] a Client handle
/// @param (backend) [in] backend name
/// @return JSON string representation of BackendProperties if succeeded, otherwise NULL. Must call daapi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_get_backend_properties(
    client: *mut Client,
    backend: *const c_char,
) -> *const c_char {
    if client.is_null() {
        return std::ptr::null();
    }
    ffi_helpers::null_pointer_check!(backend, std::ptr::null());

    if let Ok(backend_str) = CStr::from_ptr(backend).to_str() {
        match _get_backend_properties(client, backend_str) {
            Ok(props) => {
                if let Ok(props_cstr) = CString::new(props.to_string()) {
                    return props_cstr.into_raw();
                }
            }
            Err(error) => {
                eprintln!("{:?}", error);
            }
        }
    }
    std::ptr::null()
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _get_backend_configuration(
    client: *mut Client,
    backend: &str,
) -> Result<serde_json::Value> {
    (*client)
        .internal
        .get_backend_configuration::<serde_json::Value>(backend)
        .await
}
/// @brief Returns the configuration of the specified backend
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `backend` must contain a valid nul terminator at the
///   end of the string.
///
/// * `backend` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `backend` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `backend`
///
/// @param (client) [in] a Client handle
/// @param (backend) [in] backend name
/// @return JSON string representation of BackendConfiguration if succeeded, otherwise NULL. Must call daapi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_get_backend_configuration(
    client: *mut Client,
    backend: *const c_char,
) -> *const c_char {
    if client.is_null() {
        return std::ptr::null();
    }
    ffi_helpers::null_pointer_check!(backend, std::ptr::null());

    if let Ok(backend_str) = CStr::from_ptr(backend).to_str() {
        match _get_backend_configuration(client, backend_str) {
            Ok(props) => {
                if let Ok(props_cstr) = CString::new(props.to_string()) {
                    return props_cstr.into_raw();
                }
            }
            Err(error) => {
                eprintln!("{:?}", error);
            }
        }
    }
    std::ptr::null()
}

/// # Safety
///        
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _get_version(client: *mut Client) -> Result<String> {
    (*client).internal.get_service_version().await
}
/// @brief Returns the current version of the service
///
/// # Safety
///        
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// @param (client) [in] a Client handle
/// @return version string if succeeded, otherwise NULL. Must call daapi_free_string() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_get_version(client: *mut Client) -> *const c_char {
    if client.is_null() {
        return std::ptr::null();
    }
    if let Ok(version) = _get_version(client) {
        if let Ok(ver) = CString::new(version) {
            return ver.into_raw();
        }
    }
    std::ptr::null()
}

/// # Safety
#[tokio::main]
async unsafe fn _cancel_job(client: *mut Client, job_id: &str, delete_job: bool) -> Result<()> {
    (*client).internal.cancel_job(job_id, delete_job).await
}
/// @brief Cancels a job if it has not yet terminated.
///        
/// # Safety
///        
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `job_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `job_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `job_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `job_id`
///
/// @param (client) [in] a Client handle
/// @param (job_id) [in] Identifier of an existing job
/// @param (delete_job) [in] True if the job is deleted after cancellation, false otherwise.
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_cancel_job(
    client: *mut Client,
    job_id: *const c_char,
    delete_job: bool,
) -> c_int {
    if let Ok(id) = CStr::from_ptr(job_id).to_str() {
        if let Ok(()) = _cancel_job(client, id, delete_job) {
            return DAAPI_SUCCESS;
        }
    }
    DAAPI_ERROR
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _delete_job(client: *mut Client, job_id: &str) -> Result<()> {
    (*client).internal.delete_job(job_id).await
}
/// @brief Deletes a job if it has terminated.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `job_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `job_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `job_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `job_id`
///
/// @param (client) [in] a Client handle
/// @param (job_id) [in] Identifier of an existing job
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_delete_job(client: *mut Client, job_id: *const c_char) -> c_int {
    if let Ok(id) = CStr::from_ptr(job_id).to_str() {
        if let Ok(()) = _delete_job(client, id) {
            return DAAPI_SUCCESS;
        }
    }
    DAAPI_ERROR
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _list_backends(client: *mut Client) -> Result<BackendListIntermediate> {
    (*client)
        .internal
        .list_backends::<BackendListIntermediate>()
        .await
}
/// @brief Returns a list of the backends
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// @param (client) [in] A Client handle
/// @return A BackendList if succeeded, otherwise NULL. Must call daapi_free_backend_list() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_list_backends(client: *mut Client) -> *mut BackendList {
    let intermediates = match _list_backends(client) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{:?}", err);
            return std::ptr::null_mut::<BackendList>();
        }
    };
    let mut c_array = Vec::new();
    for backend in &intermediates.backends {
        c_array.push(Backend {
            name: CString::new(backend.name.clone()).unwrap().into_raw(),
            status: backend.status,
        });
    }
    let boxed_array = Box::new(BackendList {
        backends: c_array.as_mut_ptr(),
        length: c_array.len(),
    });
    std::mem::forget(c_array);
    Box::into_raw(boxed_array)
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_cli_list_backends() or related functions. Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_cli_list_backends().
///
/// @param (ptr) a ptr to BackendList
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_backend_list(ptr: *mut BackendList) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }

    unsafe {
        let array = Box::from_raw(ptr);

        for i in 0..array.length {
            let item = array.backends.add(i);
            if !(*item).name.is_null() {
                let _ = CString::from_raw((*item).name);
            }
        }
        let _ = Vec::from_raw_parts(array.backends, array.length, array.length);
    }
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _get_job_status(
    client: *mut Client,
    job_id: &str,
) -> Result<direct_access_api::models::JobStatus> {
    (*client).internal.get_job_status(job_id).await
}
/// @brief Returns the status of the specfied job.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `job_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * The memory pointed to by `outp` must have enough room to store JobStatus value.
///
/// * `job_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `job_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `job_id`
///
/// @param (client) [in] A Client handle
/// @param (job_id) [in] A job identifier
/// @param (outp) [out] JobStatus
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_get_job_status(
    client: *mut Client,
    job_id: *const c_char,
    outp: *mut JobStatus,
) -> c_int {
    ffi_helpers::null_pointer_check!(job_id, DAAPI_ERROR);
    ffi_helpers::null_pointer_check!(outp, DAAPI_ERROR);
    if let Ok(job_id_str) = CStr::from_ptr(job_id).to_str() {
        let result = match _get_job_status(client, job_id_str) {
            Ok(val) => val,
            Err(err) => {
                eprintln!("{:?}", err);
                return DAAPI_ERROR;
            }
        };

        *outp = JobStatus::from(result);
    }
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _get_job(
    client: *mut Client,
    job_id: &str,
) -> Result<direct_access_api::models::Job> {
    (*client)
        .internal
        .get_job::<direct_access_api::models::Job>(job_id)
        .await
}
/// @brief Returns metrics of the specfied job.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `job_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `job_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `job_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `job_id`
///
/// @param (client) [in] A Client handle
/// @param (job_id) [in] A job identifier
/// @return A Metrics if succeeded, otherwise NULL. Must call daapi_free_metrics() to free if no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_get_metrics(
    client: *mut Client,
    job_id: *const c_char,
) -> *mut Metrics {
    ffi_helpers::null_pointer_check!(job_id, std::ptr::null_mut::<Metrics>());
    if let Ok(job_id_str) = CStr::from_ptr(job_id).to_str() {
        if let Ok(job_details) = _get_job(client, job_id_str) {
            let c_metrics = Box::new(_to_metrics(job_details));
            return Box::into_raw(c_metrics);
        }
    }
    std::ptr::null_mut::<Metrics>()
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_cli_get_metrics(). Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_cli_get_metrics().
///
/// @param (ptr) [in] A Metrics
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_metrics(ptr: *mut Metrics) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }

    unsafe {
        let _ = Box::from_raw(ptr);
    }
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _list_jobs(client: *mut Client) -> Result<direct_access_api::models::Jobs> {
    (*client)
        .internal
        .list_jobs::<direct_access_api::models::Jobs>()
        .await
}
/// @brief Returns jobs submitted by current client in ascending order of created time by default.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// @param (client) [in] A Client handle
/// @return JobList if succeeded, otherwise NULL. Must call daapi_free_job_list() to free if no longer used.
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_list_jobs(client: *mut Client) -> *mut JobList {
    let jobs = match _list_jobs(client) {
        Ok(val) => val,
        Err(err) => {
            eprintln!("{:?}", err);
            return std::ptr::null_mut::<JobList>();
        }
    };
    let mut c_array = Vec::new();
    for job in &jobs.jobs {
        c_array.push(Job {
            id: CString::new(job.id.clone()).unwrap().into_raw(),
            status: JobStatus::from(job.status.clone()),
            program_id: ProgramId::from(job.program_id.clone()),
            metrics: _to_metrics(job.clone()),
        });
    }
    let boxed_array = Box::new(JobList {
        jobs: c_array.as_mut_ptr(),
        length: c_array.len(),
    });
    std::mem::forget(c_array);
    Box::into_raw(boxed_array)
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_cli_list_jobs() or related functions. Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, no operation is performed.
///
/// # Safety
///
/// * `ptr` must have been returned by a previous call to daapi_cli_list_jobs().
///
/// @param (ptr) [in] a ptr to JobList
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
#[no_mangle]
pub unsafe extern "C" fn daapi_free_job_list(ptr: *mut JobList) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }

    unsafe {
        let array = Box::from_raw(ptr);

        for i in 0..array.length {
            let item = array.jobs.add(i);
            if !(*item).id.is_null() {
                let _ = CString::from_raw((*item).id);
            }
        }
        let _ = Vec::from_raw_parts(array.jobs, array.length, array.length);
    }
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _run_job(client: *mut Client, payload: &str) -> Result<String> {
    let payload_json: serde_json::Value = serde_json::from_str(payload)?;
    (*client).internal.run_job(&payload_json).await
}
/// @brief Invokes a Qiskit Runtime primitive.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `payload` must contain a valid nul terminator at the
///   end of the string.
///
/// * `payload` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `payload` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `payload`
///
/// @param (client) [in] A Client handler
/// @param (payload) [in] JSON string representation of job. See Direct Access API specification for more details.
/// @return Identifier of an existing job. Must call daapi_free_string() to free if no longer used. Returns NULL if this function call is failed.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_run_job(
    client: *mut Client,
    payload: *const c_char,
) -> *const c_char {
    if client.is_null() {
        return std::ptr::null();
    }
    ffi_helpers::null_pointer_check!(payload, std::ptr::null());

    if let Ok(payload_str) = CStr::from_ptr(payload).to_str() {
        match _run_job(client, payload_str) {
            Ok(job_id) => {
                if let Ok(job_id_cstr) = CString::new(job_id) {
                    return job_id_cstr.into_raw();
                }
            }
            Err(error) => {
                eprintln!("{:?}", error);
            }
        }
    }
    std::ptr::null()
}

/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
#[tokio::main]
async unsafe fn _run_primitive(
    client: *mut Client,
    backend: &str,
    program_id: direct_access_api::models::ProgramId,
    timeout_secs: u64,
    log_level: direct_access_api::models::LogLevel,
    payload: &str,
    job_id: Option<String>,
) -> Result<direct_access_api::PrimitiveJob> {
    let payload_json: serde_json::Value = serde_json::from_str(payload)?;
    (*client)
        .internal
        .run_primitive(
            backend,
            program_id,
            timeout_secs,
            log_level,
            &payload_json,
            job_id,
        )
        .await
}
/// @brief Invokes a Qiskit Runtime primitive.
///
/// If the `job_id` is not null, the specified value is used as job identifier; if the `job_id` is null, a job identifier is automatically generated by this API client.
///
/// # Safety
///
/// * `client` must have been returned by a previous call to daapi_cli_new().
///
/// * The memory pointed to by `backend`/`payload`/`job_id` must contain a valid nul terminator at the
///   end of the string.
///
/// * `backend`/`payload`/`job_id` must be [valid] for reads of bytes up to and including the nul terminator.
///   This means in particular:
///
///     * The entire memory range of this `CStr` must be contained within a single allocated object!
///     * `backend`/`payload`/`job_id` must be non-null even for a zero-length cstr.
///
/// * The memory referenced by the returned `CStr` must not be mutated for
///   the duration of lifetime `'a`.
///
/// * The nul terminator must be within `isize::MAX` from `backend`/`payload`/`job_id`
///
/// @param (client) [in] A Client handle
/// @param (backend) [in] Name that identifies the system on which to run the job
/// @param (program_id) [in] ID of the primitive to be executed - SAMPLER or ESTIMATOR
/// @param (timeout_secs) [in] timeout in seconds
/// @param (log_level) [in] Logging level
/// @param (payload) [in] Parameters to inject into the primitive as key-value pairs. See <a href="https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/estimator_v2_schema.json">EstimatorV2 input</a> or <a href="https://github.com/Qiskit/ibm-quantum-schemas/blob/main/schemas/sampler_v2_schema.json">SamplerV2 input</a> for more details.
/// @param (job_id) [in] Optional. Specify non-null value if you want to override auto-generated job identifier.
/// @return a new PrimitiveJob if succeeded, otherwise NULL. Must call daapi_free_primitive() if a PrimitiveJob is no longer used.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_cli_run_primitive(
    client: *mut Client,
    backend: *const c_char,
    program_id: ProgramId,
    timeout_secs: c_ulong,
    log_level: LogLevel,
    payload: *const c_char,
    job_id: *const c_char,
) -> *mut PrimitiveJob {
    ffi_helpers::null_pointer_check!(backend, std::ptr::null_mut::<PrimitiveJob>());
    ffi_helpers::null_pointer_check!(payload, std::ptr::null_mut::<PrimitiveJob>());
    if let (Ok(backend_str), Ok(payload_str)) = (
        CStr::from_ptr(backend).to_str(),
        CStr::from_ptr(payload).to_str(),
    ) {
        let id: Option<String>;
        if <_ as ffi_helpers::Nullable>::is_null(&job_id) {
            id = None;
        } else if let Ok(id_str) = CStr::from_ptr(job_id).to_str() {
            id = Some(id_str.to_string());
        } else {
            return std::ptr::null_mut::<PrimitiveJob>();
        }

        if let Ok(internal) = _run_primitive(
            client,
            backend_str,
            program_id.into(),
            timeout_secs,
            log_level.into(),
            payload_str,
            id,
        ) {
            let c_job = Box::new(PrimitiveJob { internal });
            return Box::into_raw(c_job);
        }
    }
    std::ptr::null_mut::<PrimitiveJob>()
}

/// @brief Frees the memory space pointed to by `ptr`, which must have been returned by a previous call to daapi_cli_run_primitive(). Otherwise, or if ptr has already been freed, segmentation fault occurs.  If `ptr` is NULL, returns < 0.
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// @param (job) [in] A PrimitiveJob
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_free_primitive(ptr: *mut PrimitiveJob) -> c_int {
    if ptr.is_null() {
        return DAAPI_ERROR;
    }
    unsafe {
        let _ = Box::from_raw(ptr);
    };
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
#[tokio::main]
async unsafe fn _wait_for_final_state(
    job: *mut PrimitiveJob,
) -> Result<direct_access_api::models::Job> {
    (*job).internal.wait_for_final_state(None).await
}

/// @brief Polls for the job status from the API until the status is in a final state.
///
/// If `outp` is not NULL, the final state will be stored to this memory.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// * The memory pointed to by `outp` must have enough room to store JobStatus value.
///
/// @param (job) [in] A PrimitiveJob
/// @param (outp) [out] JobStatus of the final state (COMPLETED, ERROR or CANCELLED).
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_wait_for_final_state(
    job: *mut PrimitiveJob,
    outp: *mut JobStatus,
) -> c_int {
    if job.is_null() {
        return DAAPI_ERROR;
    }

    match _wait_for_final_state(job) {
        Ok(job_details) => {
            if !outp.is_null() {
                *outp = JobStatus::from(job_details.status);
            }
        }
        Err(error) => {
            eprintln!("{:?}", error);
            return DAAPI_ERROR;
        }
    }
    DAAPI_SUCCESS
}

/// @brief Returns an identifier of the job associated with a given PrimitiveJob.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// @param (job) [in] A PrimitiveJob
/// @return A job identifier if succeeded, otherwise NULL.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_get_job_id(job: *mut PrimitiveJob) -> *const c_char {
    let job_id = (*job).internal.job_id.clone();
    if let Ok(c_job_id) = CString::new(job_id) {
        return c_job_id.into_raw();
    }
    std::ptr::null()
}

/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
#[tokio::main]
async unsafe fn _prim_is_running(job: *mut PrimitiveJob) -> Result<bool> {
    (*job).internal.is_running().await
}
/// @brief Returns whether the job is actively running.
///
/// If `outp` is not NULL, the boolean value (running or not) will be stored to this memory.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// * The memory pointed to by `outp` must have enough room to store bool value.
///
/// @param (job) [in] A PrimitiveJob
/// @param (outp) [out] `true` if the job is running, false otherwise.
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_is_running(job: *mut PrimitiveJob, outp: *mut bool) -> c_int {
    if job.is_null() {
        return DAAPI_ERROR;
    }

    match _prim_is_running(job) {
        Ok(is_running) => {
            if !outp.is_null() {
                *outp = is_running;
            }
        }
        Err(error) => {
            eprintln!("{:?}", error);
            return DAAPI_ERROR;
        }
    }
    DAAPI_SUCCESS
}

/// @brief Returns `true` if the status is in a final state.
///
/// If `outp` is not NULL, the boolean value (in final state or not) will be stored to this memory.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// * The memory pointed to by `outp` must have enough room to store bool value.
///
/// @param (job) [in] A PrimitiveJob
/// @param (outp) [out] `true` if the job is in a final state, false otherwise.
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_is_in_final_state(
    job: *mut PrimitiveJob,
    outp: *mut bool,
) -> c_int {
    if job.is_null() {
        return DAAPI_ERROR;
    }

    match _prim_is_running(job) {
        Ok(is_running) => {
            if !outp.is_null() {
                *outp = !is_running;
            }
        }
        Err(error) => {
            eprintln!("{:?}", error);
            return DAAPI_ERROR;
        }
    }
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
#[tokio::main]
async unsafe fn _prim_cancel(job: *mut PrimitiveJob, delete_job: bool) -> Result<()> {
    (*job).internal.cancel(delete_job).await
}
/// @brief Cancels a job if it has not yet terminated.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// @param (job) [in] A PrimitiveJob
/// @param (delete_job) [in] True if the job is deleted after cancellation, false otherwise.
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_cancel(job: *mut PrimitiveJob, delete_job: bool) -> c_int {
    if job.is_null() {
        return DAAPI_ERROR;
    }

    if let Err(error) = _prim_cancel(job, delete_job) {
        eprintln!("{:?}", error);
        return DAAPI_ERROR;
    }
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
#[tokio::main]
async unsafe fn _prim_delete(job: *mut PrimitiveJob) -> Result<()> {
    (*job).internal.delete().await
}
/// @brief Deletes a job if it has terminated.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// @param (job) [in] A PrimitiveJob
/// @return DAAPI_SUCCESS(0) if succeeded, otherwise < 0.
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_delete(job: *mut PrimitiveJob) -> c_int {
    if job.is_null() {
        return DAAPI_ERROR;
    }

    if let Err(error) = _prim_delete(job) {
        eprintln!("{:?}", error);
        return DAAPI_ERROR;
    }
    DAAPI_SUCCESS
}

/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
#[tokio::main]
async unsafe fn _prim_get_result<T: DeserializeOwned>(job: *mut PrimitiveJob) -> Result<T> {
    (*job).internal.get_result::<T>().await
}
/// @brief Returns the results of the job.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// @param (job) [in] A PrimitiveJob
/// @return Log contents
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_get_result_as_string(job: *mut PrimitiveJob) -> *const c_char {
    if job.is_null() {
        return std::ptr::null();
    }

    match _prim_get_result::<serde_json::Value>(job) {
        Ok(json) => {
            if let Ok(json_str) = CString::new(json.to_string()) {
                return json_str.into_raw();
            }
        }
        Err(error) => {
            eprintln!("{:?}", error);
        }
    }
    std::ptr::null()
}

/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
#[tokio::main]
async unsafe fn _prim_get_logs(job: *mut PrimitiveJob) -> Result<String> {
    (*job).internal.get_logs().await
}
/// @brief Returns the logs of the job.
///
/// # Safety
///
/// * `job` must have been returned by a previous call to daapi_cli_run_primitive().
///
/// @param (job) [in] A PrimitiveJob
/// @return Log contents
/// @version 0.1.0
#[no_mangle]
pub unsafe extern "C" fn daapi_prim_get_logs(job: *mut PrimitiveJob) -> *const c_char {
    if job.is_null() {
        return std::ptr::null();
    }

    match _prim_get_logs(job) {
        Ok(logs) => {
            if let Ok(logs_str) = CString::new(logs) {
                return logs_str.into_raw();
            }
        }
        Err(error) => {
            eprintln!("{:?}", error);
        }
    }
    std::ptr::null()
}
