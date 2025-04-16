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
use anyhow::{Result};
// use retry_policies::policies::ExponentialBackoff;
// use retry_policies::Jitter;
// use serde_json::json;
use pasqal_cloud_api::{
    Client, ClientBuilder, GetAuthInfoResponse
};
use std::collections::HashMap;
use std::env;
// use std::str::FromStr;
// use std::time::Duration;
use uuid::Uuid;

// python binding
use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;

// c binding
// use crate::consts::{QRMI_ERROR, QRMI_SUCCESS};
// use std::ffi::CStr;
// use std::ffi::CString;
// use std::os::raw::{c_char, c_int};

/// QRMI implementation for Pasqal Cloud
#[pyclass]
pub struct PasqalCloud {
    pub(crate) api_client: Client,
}

#[pymethods]
impl PasqalCloud {
    /// Constructs a QRMI to access Pasqal Cloud Service
    ///
    /// # Environment variables
    ///
    /// * `QRMI_PASQAL_CLOUD_PROJECT_ID`: Pasqal Cloud Project ID to access the QPU
    /// * `QRMI_PASQAL_CLOUD_AUTH_TOKEN`: Pasqal Cloud Auth Token
    /// Let's hardcode the rest for now
    #[new]
    pub fn new() -> Self {
        // Check to see if the environment variables required to run this program are set.
        let project_id = env::var("QRMI_PASQAL_CLOUD_PROJECT_ID").expect("QRMI_PASQAL_CLOUD_PROJECT_ID");
        let auth_token = env::var("QRMI_PASQAL_CLOUD_AUTH_TOKEN").expect("QRMI_PASQAL_CLOUD_AUTH_TOKEN");
        Self {
            api_client: ClientBuilder::new(auth_token, project_id).build().unwrap(),
        }
    }

    /// Python binding of QRMI is_accessible() function.
    #[pyo3(name = "is_accessible")]
    fn pyfunc_is_accessible(&mut self, id: &str) -> PyResult<bool> {
        Ok(self.is_accessible(id))
    }

    /// Python binding for testing
    #[pyo3(name = "get_auth_info")]
    fn pyfunc_get_auth_info(&mut self) -> PyResult<String> {
        self._get_auth_info()
    }

    /// Python binding of QRMI acquire() function.
    #[pyo3(name = "acquire")]
    fn pyfunc_acquire(&mut self, id: &str) -> PyResult<String> {
        match self.acquire(id) {
            Ok(v) => Ok(v),
            Err(v) => Err(v.into()),
        }
    }

    /// Python binding of QRMI release() function.
    #[pyo3(name = "release")]
    fn pyfunc_release(&mut self, id: &str) -> PyResult<()> {
        match self.release(id) {
            Ok(()) => Ok(()),
            Err(v) => Err(v.into()),
        }
    }

    /// Python binding of QRMI task_start() function.
    #[pyo3(name = "task_start")]
    fn pyfunc_task_start(&mut self, payload: Payload) -> PyResult<String> {
        match self.task_start(payload) {
            Ok(v) => Ok(v),
            Err(v) => Err(v.into()),
        }
    }

    /// Python binding of QRMI task_stop() function.
    #[pyo3(name = "task_stop")]
    fn pyfunc_task_stop(&mut self, task_id: &str) -> PyResult<()> {
        match self.task_stop(task_id) {
            Ok(()) => Ok(()),
            Err(v) => Err(v.into()),
        }
    }

    /// Python binding of QRMI task_status() function.
    #[pyo3(name = "task_status")]
    fn pyfunc_task_status(&mut self, task_id: &str) -> PyResult<TaskStatus> {
        match self.task_status(task_id) {
            Ok(v) => Ok(v),
            Err(v) => Err(v.into()),
        }
    }

    /// Python binding of QRMI task_result() function.
    #[pyo3(name = "task_result")]
    fn pyfunc_task_result(&mut self, task_id: &str) -> PyResult<TaskResult> {
        match self.task_result(task_id) {
            Ok(v) => Ok(v),
            Err(v) => Err(v.into()),
        }
    }

    /// Python binding of QRMI target() function.
    #[pyo3(name = "target")]
    fn pyfunc_target(&mut self, id: &str) -> PyResult<Target> {
        match self.target(id) {
            Ok(v) => Ok(v),
            Err(v) => Err(v.into()),
     
        }
    }

    /// Python binding of QRMI metadata() function.
    #[pyo3(name = "metadata")]
    fn pyfunc_metadata(&mut self) -> PyResult<HashMap<String, String>> {
        let mut metadata: HashMap<String, String> = HashMap::new();
        Ok(metadata)
    }
}

impl Default for PasqalCloud {
    fn default() -> Self {
        Self::new()
    }
}
/// QuantumResource Trait implementation for PasqalCloud
impl PasqalCloud {
    /// Wrapper of async call for QRMI is_accessible() function.
    #[tokio::main]
    async fn _is_accessible(&mut self, id: &str) -> bool {
        return true;
    }

    /// Wrapper of async call for QRMI task_start() function.
    #[tokio::main]
    async fn _task_start(&mut self, payload: Payload) -> Result<String> {
        return Ok("started".to_string());
    }

    /// Wrapper of async call for QRMI task_stop() function.
    #[tokio::main]
    async fn _task_stop(&mut self, task_id: &str) -> Result<()> {
        return Ok(())
    }

    /// Wrapper of async call for QRMI task_status() function.
    #[tokio::main]
    async fn _task_status(&mut self, task_id: &str) -> Result<TaskStatus> {
        return Ok(TaskStatus::Completed);
    }

    /// Wrapper of async call for QRMI task_result() function.
    #[tokio::main]
    async fn _task_result(&mut self, task_id: &str) -> Result<TaskResult> {
        return Ok(TaskResult {
            value: "works fine".to_string(),
        })
    }

    /// Wrapper of async call for QRMI target() function.
    #[tokio::main]
    async fn _target(&mut self, id: &str) -> Result<Target> {
        return Ok(Target {
            value: "target".to_string(),
        })
    }

    #[tokio::main]
    async fn _get_auth_info(&mut self) -> PyResult<String> {
        match self.api_client.get_auth_info().await {
            Ok(v) => Ok(v),
            Err(v) => Err(v.into()),
        }
    }
}

impl QuantumResource for PasqalCloud {
    fn is_accessible(&mut self, id: &str) -> bool {
        self._is_accessible(id)
    }

    fn acquire(&mut self, _id: &str) -> Result<String> {
        // Pasqal Cloud does not support session concept, so simply returns dummy ID for now.
        Ok(Uuid::new_v4().to_string())
    }

    fn release(&mut self, _id: &str) -> Result<()> {
        // Pasqal Cloud does not support session concept, so simply ignores
        Ok(())
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
        let mut metadata: HashMap<String, String> = HashMap::new();
        metadata
    }
}
