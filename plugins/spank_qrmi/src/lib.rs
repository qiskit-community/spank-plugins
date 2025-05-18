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
//
use eyre::WrapErr;
use slurm_spank::{Context, Plugin, SpankHandle, SpankOption, SLURM_VERSION_NUMBER, SPANK_PLUGIN};
use tracing::{info, error};

use std::error::Error;
use std::process;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

mod models;
use self::models::{QRMIResource, QRMIResources, ResourceType};

use qrmi::ibm::{IBMDirectAccess, IBMQiskitRuntimeService};
use qrmi::QuantumResource;

const SLURM_BATCH_SCRIPT: u32 = 0xfffffffb;

// spank_qrmi plugin
//
// All spank plugins must define this macro for the Slurm plugin loader.
SPANK_PLUGIN!(b"spank_qrmi", SLURM_VERSION_NUMBER, SpankQrmi);

#[derive(Default)]
struct SpankQrmi {
    qpu_names: Option<Vec<String>>,
    qpu_types: Option<Vec<ResourceType>>,
    acquisition_tokens: Option<Vec<String>>,
}

/// Log entering function
macro_rules! enter {
    () => {
        info!("PID = {}, UID = {}", process::id(), unsafe {
            libc::getuid()
        });
    };
}

/// Dump Spank context
macro_rules! dump_context {
    ($spank:expr) => {
        if let Ok(result) = $spank.job_id() {
            info!("S_JOB_ID = {}", result);
        } else {
            info!("S_JOB_ID =");
        }
        if let Ok(result) = $spank.job_stepid() {
            info!("S_JOB_STEPID = {:x}", result);
        } else {
            info!("S_JOB_STEPID =");
        }
        info!("S_JOB_ARGV = {:#?}", $spank.job_argv().unwrap_or(vec!()));
        info!(
            "S_PLUGIN_ARGV = {:#?}",
            $spank.plugin_argv().unwrap_or(vec!())
        );
    };
}

unsafe impl Plugin for SpankQrmi {
    /// slurm_spank_init
    ///
    /// Called just after plugins are loaded.
    ///
    /// In remote context, this is just after job step is initialized. This
    /// function is called before any plugin option processing.
    ///
    /// This plugin registers '--qpu=names' option to allow users to specify
    /// quantum resources to be used in the job.
    fn init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);
        }
        // Register the --qpu=names option
        match spank.context()? {
            Context::Local | Context::Remote | Context::Allocator => {
                spank
                    .register_option(
                        SpankOption::new("qpu")
                            .takes_value("names")
                            .usage("Comma separated list of QPU resources to use."),
                    )
                    .wrap_err("Failed to register --qpu=names option")?;
            }
            _ => {}
        }
        Ok(())
    }

    /// slurm_spank_init_post_opt
    ///
    /// Called at the same point as slurm_spank_init, but after all user options
    /// to the plugin have been processed.
    ///
    /// The reason that the init and init_post_opt callbacks are separated is so
    /// that plugins can process system-wide options specified in plugstack.conf
    /// in the init callback, then process user options, and finally take some
    /// action in slurm_spank_init_post_opt if necessary. In the case of a
    /// heterogeneous job, slurm_spank_init is invoked once per job component.
    ///
    /// This plugin invokes QRMI.acquire() to obtain access to Quantum resource, and
    /// store the returned acquisition tokens to memory.
    fn init_post_opt(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        // Check if the option was set
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);
        } else {
            // skip if context != remote
            return Ok(());
        }

        if let Ok(step_id) = spank.job_stepid() {
            // skip if this is slurm task steps
            if step_id != SLURM_BATCH_SCRIPT {
                return Ok(());
            }
        }

        let qpu_option = spank
            .get_option_value("qpu")
            .wrap_err("Failed to read --qpu=names option")?
            .map(|s| s.to_string());
        if qpu_option.is_none() {
            // do nothing if not qpu job
            return Ok(());
        }

        let binding = qpu_option.unwrap();
        let qpu_names: Vec<&str> = binding.split(",").map(|l| l.trim()).collect();
        info!("qpu names = {:#?}", qpu_names);

        let plugin_argv = spank.plugin_argv().unwrap_or_default();
        if plugin_argv.len() != 1 {
            return Ok(());
        }
        let f = File::open(plugin_argv[0]).expect("qrmi_config.json not found");
        let mut buf_reader = BufReader::new(f);
        let mut config_json_str = String::new();
        buf_reader.read_to_string(&mut config_json_str)?;
        let config = serde_json::from_str::<QRMIResources>(&config_json_str).unwrap();

        let mut config_map: HashMap<String, QRMIResource> = HashMap::new();
        for qrmi in config.resources {
            config_map.insert(qrmi.name.clone(), qrmi);
        }

        let mut avail_names: Vec<String> = vec!();
        let mut avail_types: Vec<String> = vec!();
        let mut types: Vec<ResourceType> = vec!();
        let mut acquisition_tokens: Vec<String> = Vec::new();
        for qpu_name in qpu_names {
            if let Some(qrmi) = config_map.get(qpu_name) {
                info!(
                    "qpu = {}, type = {:#?} env = {:#?}",
                    qpu_name, qrmi.r#type, qrmi.environment
                );

                // Set environment variables specified in config file.
                for (key, value) in &qrmi.environment {
                    // set to job's envronment
                    spank.setenv(format!("{qpu_name}_{key}"), value, true)?;
                    // set to the current process for subsequent QRMI.acquire() call
                    env::set_var(format!("{qpu_name}_{key}"), value);
                }

                match qrmi.r#type {
                    ResourceType::IBMDirectAccess => {
                        let mut instance = IBMDirectAccess::new(qpu_name);
                        let token: Option<String> = match instance.acquire() {
                            Ok(v) => Some(v),
                            Err(err) => {
                                error!("qrmi.acquire() failed: {:#?}", err);
                                None
                            },
                        };

                        if let Some(acquisition_token) = token {
                            info!("acquisition token = {}", acquisition_token);
                            spank.setenv(format!("{qpu_name}_QRMI_IBM_DA_SESSION_ID"), &acquisition_token, true)?;
                            avail_names.push(qpu_name.to_string());
                            avail_types.push(qrmi.r#type.as_str().to_string());
                            types.push(qrmi.r#type.clone());
                            acquisition_tokens.push(acquisition_token);
                        }
                    }
                    ResourceType::QiskitRuntimeService => {
                        let mut instance = IBMQiskitRuntimeService::new(qpu_name);
                        let token: Option<String> = match instance.acquire() {
                            Ok(v) => Some(v),
                            Err(err) => {
                                error!("qrmi.acquire() failed: {:#?}", err);
                                None
                            },
                        };

                        if let Some(acquisition_token) = token {
                            info!("acquisition token = {}", acquisition_token);
                            spank.setenv(format!("{qpu_name}_QRMI_IBM_QRS_SESSION_ID"), &acquisition_token, true)?;
                            avail_names.push(qpu_name.to_string());
                            avail_types.push(qrmi.r#type.as_str().to_string());
                            types.push(qrmi.r#type.clone());
                            acquisition_tokens.push(acquisition_token);
                        }
                    }
                    _ => {
                        // skip unsupported type
                    }
                }
            }
        }
        spank.setenv("SLURM_JOB_QPU_RESOURCES", avail_names.join(","), true)?;
        spank.setenv("SLURM_JOB_QPU_TYPES", avail_types.join(","), true)?;
        self.qpu_names = Some(avail_names);
        self.qpu_types = Some(types);
        self.acquisition_tokens = Some(acquisition_tokens);

        Ok(())
    }

    /// slurm_spank_exit
    ///
    /// Called once just before slurmstepd exits in remote context. In local
    /// context, called before srun exits.
    ///
    /// This plugin invokes QRMI.release() to release Quantum resource.
    fn exit(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        if spank.context()? == Context::Remote {
            dump_context!(spank);

            if let (Some(names), Some(types), Some(tokens)) = (
                self.qpu_names.clone(),
                self.qpu_types.clone(),
                self.acquisition_tokens.clone(),
            ) {
                for (index, name) in names.iter().enumerate() {
                    let res_type = &types[index];
                    let token = &tokens[index];
                    info!("releasing {}, {:#?}, {}", name, res_type, token);
                    match res_type {
                        ResourceType::IBMDirectAccess => {
                            let mut instance = IBMDirectAccess::new(name);
                            let _ = instance.release(token);
                        }
                        ResourceType::QiskitRuntimeService => {
                            let mut instance = IBMQiskitRuntimeService::new(name);
                            let _ = instance.release(token);
                        }
                        _ => {
                            // skip unsupported type
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Called for each task just before execve (2).
    ///
    /// If you are restricting memory with cgroups, memory allocated here will be
    /// in the job's cgroup. (remote context only)
    fn task_init(&mut self, spank: &mut SpankHandle) -> Result<(), Box<dyn Error>> {
        enter!();
        dump_context!(spank);
        if let Ok(result) = spank.job_env() {
            // dump job environment variables for development 
            info!("{:#?}", result);
        }
        Ok(())
    }
}
