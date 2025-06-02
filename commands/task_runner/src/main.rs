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

#![allow(unused_imports)]
use eyre::{eyre, WrapErr};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::{thread, time};

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use clap::builder::TypedValueParser as _;
use clap::{Parser, Subcommand, ValueEnum};

use qrmi::ibm::{IBMDirectAccess, IBMQiskitRuntimeService};
use qrmi::pasqal::PasqalCloud;
use qrmi::{models::Payload, models::TaskStatus, QuantumResource};

static IS_RUNNING: AtomicBool = AtomicBool::new(true);

const POLLING_INTERVAL: u64 = 1000;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
#[allow(dead_code)]
/// Qiskit Primitive types
pub enum PrimitiveType {
    /// Estimator
    Estimator,
    /// Sampler
    Sampler,
}
impl PrimitiveType {
    fn as_str(&self) -> &str {
        match self {
            PrimitiveType::Estimator => "estimator",
            PrimitiveType::Sampler => "sampler",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Subcommand)]
#[allow(dead_code)]
/// QRMI resource types
pub enum ResourceType {
    /// IBM Direct Access
    IBMDirectAccess {
        /// Qiskit primitive input
        input: String,
        /// Qiskit primitive type
        program_id: PrimitiveType,
    },
    /// IBM Qiskit Runtime Service
    QiskitRuntimeService {
        /// Qiskit primitive input
        input: String,
        /// Qiskit primitive type
        program_id: PrimitiveType,
    },
    /// Pasqal Cloud
    PasqalCloud {
        /// Pulser sequence
        sequence: String,
        /// Number of times the pulser sequence is repeated.
        job_runs: i32,
    },
}
impl ResourceType {
    fn new(qpu_type: &str, args: Args) -> Result<Self, Box<dyn std::error::Error>> {
        if qpu_type == "direct-access" {
            let program_id = match &args.program_id {
                Some(v) => v.clone(),
                None => {
                    return Err(
                        eyre!(
                            "Missing argument: {}. Usage: qrmi_task_runner --input <file> --program-id <type> <QPU_NAME>",
                            "--program-id"
                        ).into()
                    );
                }
            };
            Ok(Self::IBMDirectAccess { input: args.input, program_id })
        } else if qpu_type == "qiskit-runtime-service" {
            let program_id = match &args.program_id {
                Some(v) => v.clone(),
                None => {
                    return Err(
                        eyre!(
                            "Missing argument: {}. Usage: qrmi_task_runner --input <file> --program-id <type> <QPU_NAME>",
                            "--program-id"
                        ).into()
                    );
                }
            };
            Ok(Self::QiskitRuntimeService { input: args.input, program_id })
        } else if qpu_type == "pasqal-cloud" {
            let job_runs = match &args.job_runs {
                Some(v) => *v,
                None => {
                    return Err(
                        eyre!(
                            "Missing argument: {}. Usage: qrmi_task_runner --input <file> --job-runs <count> <QPU_NAME>",
                            "--job-runs"
                        ).into()
                    );
                }
            };
            Ok(Self::PasqalCloud { sequence: args.input, job_runs })
        } else {
            Err(
                eyre!(
                    "Resource type {} is not supported. [supported types: direct-access, qiskit-runtime-service, pasqal-cloud]",
                    qpu_type,
                ).into()
            )
        }
    }
    #[allow(dead_code)]
    fn as_str(&self) -> &str {
        match self {
            ResourceType::IBMDirectAccess { .. } => "direct-access",
            ResourceType::QiskitRuntimeService { .. } => "qiskit-runtime-service",
            ResourceType::PasqalCloud { .. } => "pasqal-cloud",
        }
    }
    fn to_payload(&self) -> Option<Payload> {
        match self {
            ResourceType::IBMDirectAccess { input, program_id }
            | ResourceType::QiskitRuntimeService { input, program_id } => {
                let f = File::open(input).unwrap_or_else(|_| panic!("{} not found", input));
                let mut buf_reader = BufReader::new(f);
                let mut contents = String::new();
                buf_reader.read_to_string(&mut contents).ok()?;
                Some(Payload::QiskitPrimitive {
                    input: contents,
                    program_id: program_id.as_str().to_string(),
                })
            }
            ResourceType::PasqalCloud { sequence, job_runs } => {
                let f = File::open(sequence).unwrap_or_else(|_| panic!("{} not found", sequence));
                let mut buf_reader = BufReader::new(f);
                let mut contents = String::new();
                buf_reader.read_to_string(&mut contents).ok()?;
                Some(Payload::PasqalCloud {
                    sequence: contents,
                    job_runs: *job_runs,
                })
            }
        }
    }
    fn create_qrmi(&self, qpu_name: &str) -> Box<dyn QuantumResource> {
        match self {
            ResourceType::IBMDirectAccess { .. } => Box::new(IBMDirectAccess::new(qpu_name)),
            ResourceType::QiskitRuntimeService { .. } => {
                Box::new(IBMQiskitRuntimeService::new(qpu_name))
            }
            ResourceType::PasqalCloud { .. } => Box::new(PasqalCloud::new(qpu_name)),
        }
    }
}

#[derive(Parser, Debug, Clone)]
#[command(version = "0.1.0")]
#[command(about = "qrmi_task_runner - Command to run a QRMI task")]
struct Args {
    /// QPU resource name.
    #[arg(short, long, value_name = "name")]
    qpu_name: String,

    #[arg(short, long, value_name = "file")]
    /// Input to QPU resource. Parameters to inject into the primitive for direct-access or qiskit-runtime-service QPU resource. Pulser sequence for pasqal-cloud QPU resource.
    input: String,

    #[arg(long, value_name = "type")]
    /// ID of the primitive to be executed. Required for direct-access or qiskit-runtime-service QPU resource.
    program_id: Option<PrimitiveType>,

    #[arg(long, value_name = "counts")]
    /// Number of times the pulser sequence is repeated. Required for pasqal-cloud QPU resource.
    job_runs: Option<i32>,

    /// Write output to <file> instead of stdout.
    #[arg(short, long, value_name = "file")]
    output: Option<String>,
}

// Handle signals, and cancel QPU job if SIGTERM is received.
async fn handle_signals(mut signals: Signals) {
    while let Some(signal) = signals.next().await {
        // To cancel a job, invoke scancel without --signal option. This will send
        // first a SIGCONT to all steps to eventually wake them up followed by a
        // SIGTERM, then wait the KillWait duration defined in the slurm.conf file
        // and finally if they have not terminated send a SIGKILL.
        match signal {
            SIGCONT | SIGTERM => {
                // cancel QPU job
                IS_RUNNING.store(false, Ordering::SeqCst);
            }
            // only registered sinals come
            _ => unreachable!(),
        }
    }
}

// Create the specified file and write the given data to it.
fn write_to_file(filename: &String, data: &[u8]) {
    if let Ok(mut f) = File::create(filename) {
        match f.write_all(data) {
            Ok(()) => {
                let _ = f.flush();
            }
            Err(e) => {
                eprintln!("Error: Failed to write output. reason = {:?}", e);
            }
        }
    }
}

// Check to see if the specified file can be created, written and truncated.
// Exit this program immediately if failed.
fn check_file_argument(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .is_err()
    {
        return Err(eyre!("{} cannot be created.", path).into());
    }
    Ok(())
}

fn find_qpu_type(
    qpu_resources: Vec<&str>,
    qpu_types: Vec<&str>,
    qpu_name: String,
) -> Option<String> {
    if let Some(index) = qpu_resources.iter().position(|&r| r == qpu_name) {
        return Some(qpu_types[index].to_string());
    }
    None
}

// Convert SRUN_DEBUG environment value to RUST_LOG value
fn to_rust_loglevel(srun_debug: &str) -> &str {
    match srun_debug.parse::<i32>() {
        Ok(level) => match level {
            // --quiet
            2 => "error",
            // default
            3 => "info",
            // --verbose
            4 => "debug",
            // -vv or more
            n if n >= 5 => "debug",
            // default is Info as same as srun
            _ => "info",
        },
        Err(_) => "info",
    }
}

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Before executing a quantum job, check to see if the specified
    // file can be created, and inform to user if it cannot be written. This is
    // to prevent file writing errors after a long job execution.
    if let Some(ref output_file) = args.output {
        check_file_argument(output_file)?;
    }

    if let Ok(srun_debug) = env::var("SRUN_DEBUG") {
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or(to_rust_loglevel(&srun_debug)),
        )
        .init();
    } else {
        // use default
        env_logger::init();
    }

    let envvar_qpu_names = match env::var("SLURM_JOB_QPU_RESOURCES") {
        Ok(v) => v,
        Err(err) => {
            return Err(
                eyre!(
                    "The environment variable `SLURM_JOB_QPU_RESOURCES` is not set and as such configuration could not be loaded. reason = {}",
                    err)
                .into()
            );
        }
    };
    let qpu_names: Vec<&str> = envvar_qpu_names.split(',').collect();

    let envvar_qpu_types = match env::var("SLURM_JOB_QPU_TYPES") {
        Ok(v) => v,
        Err(err) => {
            return Err(
                eyre!(
                    "The environment variable `SLURM_JOB_QPU_TYPES` is not set and as such configuration could not be loaded. reason = {}",
                    err)
                .into()
            );
        }
    };
    let qpu_types: Vec<&str> = envvar_qpu_types.split(',').collect();

    let qpu_name = args.qpu_name.clone();
    let res_type: ResourceType;
    if let Some(qpu_type) = find_qpu_type(qpu_names, qpu_types, qpu_name.clone()) {
        res_type = ResourceType::new(&qpu_type, args.clone())?;
    } else {
        return Err(eyre!("{} is not specified in --qpu option", qpu_name).into());
    }

    let payload = res_type.to_payload().unwrap();
    let mut qrmi = res_type.create_qrmi(&qpu_name);

    // setup signal handler for slurm, and start it
    let signals = Signals::new([SIGTERM, SIGCONT])?;
    let handle = signals.handle();
    let signals_task = tokio::spawn(handle_signals(signals));

    // start a task
    let job_id = qrmi.task_start(payload).await?;
    println!("Task ID: {}", job_id);

    // Poll the task status until it progresses to a final state such as TaskStatus::Completed.
    let mut succeeded = false;
    let one_sec = time::Duration::from_millis(POLLING_INTERVAL);
    while IS_RUNNING.load(Ordering::SeqCst) {
        match qrmi.task_status(&job_id).await {
            Ok(status) => {
                if matches!(status, TaskStatus::Completed) {
                    succeeded = true;
                    break;
                } else if matches!(status, TaskStatus::Failed | TaskStatus::Cancelled) {
                    eprintln!("{:#?}", status);
                    break;
                }
            }
            Err(err) => {
                eprintln!(
                    "Error: Failed to get task status. reason = {}. Retrying.",
                    err
                );
            }
        }
        thread::sleep(one_sec);
    }

    // write output if task was succeeded
    if succeeded {
        match qrmi.task_result(&job_id).await {
            Ok(result) => {
                if let Some(output_file) = args.output {
                    write_to_file(&output_file, result.value.as_bytes());
                    println!("Wrote output to {}.", output_file);
                } else {
                    println!("{}", result.value);
                }
            }
            Err(err) => {
                eprintln!("Error: Failed to get result. reason = {}", err);
            }
        }
    }

    // cleanup quantum task
    let _ = qrmi.task_stop(&job_id).await;

    // shutdown signal handler
    handle.close();
    signals_task.await?;

    Ok(())
}
