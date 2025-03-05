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
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

use retry_policies::policies::ExponentialBackoff;
use retry_policies::Jitter;
use std::env;
use std::time::Duration;

use futures::stream::StreamExt;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use clap::builder::TypedValueParser as _;
use clap::Parser;

use direct_access_api::{
    models::JobStatus, models::LogLevel, AuthMethod, ClientBuilder, PrimitiveJob,
};

#[derive(Parser, Debug)]
#[command(version = "0.1.0")]
#[command(about = "QRUN - Command to run Qiskit Primitive jobs")]
struct Args {
    /// Qiskit Primitive Unified Bloc(PUB)s file.
    input: String,

    /// Result output file.
    #[arg(short, long)]
    results: Option<String>,

    /// HTTP request timeout in seconds.
    #[arg(long, default_value_t = 60)]
    http_timeout: u64,
}

// Handle signals, and cancel QPU job if SIGTERM is received.
#[cfg(feature = "job_cleanup")]
async fn handle_signals(mut signals: Signals, job: PrimitiveJob) {
    while let Some(signal) = signals.next().await {
        // To cancel a job, invoke scancel without --signal option. This will send
        // first a SIGCONT to all steps to eventually wake them up followed by a
        // SIGTERM, then wait the KillWait duration defined in the slurm.conf file
        // and finally if they have not terminated send a SIGKILL.
        match signal {
            SIGTERM => {
                // cancel QPU job
                let _ = job.cancel(false).await;
                // Submitted job has run longer than the allocated time limit user specified
                // when submitting it, causing the system to automatically terminate the job.
                // In this case, multiple SIGTERM signals are sent to this handler.
                // Break this loop if QPU job is already in final state to avoid issuing
                // multiple cancel requests to Quantun backend.
                if job.is_in_final_state().await.unwrap_or(false) {
                    break;
                }
            }
            SIGCONT => {
                // Nothing to be done by qrun.
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
                eprintln!("{:?}", e);
            }
        }
    }
}

// Check to see if the specified file can be created, written and truncated.
// Exit this program immediately if failed.
fn check_file_argument(path: &str) {
    if OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .is_err()
    {
        eprintln!("File cannot be created at: {}", path);
        std::process::exit(1)
    }
}

// Convert SRUN_DEBUG environment value to DA LogLevel
fn get_log_level(srun_debug: &str) -> LogLevel {
    match srun_debug.parse::<i32>() {
        Ok(level) => match level {
            // --quiet
            2 => LogLevel::Error,
            // default
            3 => LogLevel::Info,
            // --verbose
            4 => LogLevel::Debug,
            // -vv or more
            n if n >= 5 => LogLevel::Debug,
            // default is Info as same as srun
            _ => LogLevel::Info,
        },
        Err(_) => LogLevel::Info,
    }
}

// Convert SRUN_DEBUG environment value to RUST_LOG value
fn get_rust_log_level(srun_debug: &str) -> &str {
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
    if let Some(ref results_file) = args.results {
        check_file_argument(results_file);
    }

    // Check to see if the environment variables required to run this program are set.
    let backend_name = env::var("IBMQRUN_BACKEND").expect("IBMQRUN_BACKEND");
    let program_id = env::var("IBMQRUN_PRIMITIVE").expect("IBMQRUN_PRIMITIVE");
    let daapi_endpoint = env::var("IBMQRUN_DAAPI_ENDPOINT").expect("IBMQRUN_DAAPI_ENDPOINT");
    let aws_access_key_id =
        env::var("IBMQRUN_AWS_ACCESS_KEY_ID").expect("IBMQRUN_AWS_ACCESS_KEY_ID");
    let aws_secret_access_key =
        env::var("IBMQRUN_AWS_SECRET_ACCESS_KEY").expect("IBMQRUN_AWS_SECRET_ACCESS_KEY");
    let s3_endpoint = env::var("IBMQRUN_S3_ENDPOINT").expect("IBMQRUN_S3_ENDPOINT");
    let s3_bucket = env::var("IBMQRUN_S3_BUCKET").expect("IBMQRUN_S3_BUCKET");
    let s3_region = env::var("IBMQRUN_S3_REGION").expect("IBMQRUN_S3_REGION");

    // Slurm's time limit is wall clock time, and DA API's timeout_secs is total quantum time.
    // By specifying the time limit of Slurm as the timeout_secs of the DA API, we can avoid
    // timeout in DA API side.
    let timeout = env::var("IBMQRUN_TIMEOUT_SECONDS").expect("IBMQRUN_TIMEOUT_SECONDS");
    let timeout_secs = timeout.parse::<u64>().expect("IBMQRUN_TIMEOUT_SECONDS");

    let job_id: Option<String>;
    if let Ok(envvar) = env::var("IBMQRUN_JOB_ID") {
        job_id = Some(envvar);
    } else {
        job_id = None;
    }

    let log_level: LogLevel;
    if let Ok(srun_debug) = env::var("SRUN_DEBUG") {
        // Direct Access - service side log level
        log_level = get_log_level(&srun_debug);
        // Direct Access - client side log level
        env_logger::Builder::from_env(
            env_logger::Env::default().default_filter_or(get_rust_log_level(&srun_debug)),
        )
        .init();
    } else {
        // Info is default as same as srun's default
        log_level = LogLevel::Info;
        env_logger::init();
    }

    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
        .jitter(Jitter::Bounded)
        .base(2)
        .build_with_max_retries(5);

    let mut auth_method = AuthMethod::None;
    if let (Some(apikey), Some(service_crn), Some(iam_endpoint_url)) = (
        env::var("IBMQRUN_IAM_APIKEY").ok(),
        env::var("IBMQRUN_SERVICE_CRN").ok(),
        env::var("IBMQRUN_IAM_ENDPOINT").ok(),
    ) {
        auth_method = AuthMethod::IbmCloudIam {
            apikey,
            service_crn,
            iam_endpoint_url,
        };
    }

    #[cfg(feature = "ibmcloud_appid_auth")]
    if let AuthMethod::None = auth_method {
        if let (Some(username), Some(password)) = (
            env::var("IBMQRUN_APPID_CLIENT_ID").ok(),
            env::var("IBMQRUN_APPID_SECRET").ok(),
        ) {
            auth_method = AuthMethod::IbmCloudAppId { username, password };
        }
    }

    let client = ClientBuilder::new(daapi_endpoint)
        .with_timeout(Duration::from_secs(args.http_timeout))
        .with_retry_policy(retry_policy)
        .with_s3bucket(
            aws_access_key_id,
            aws_secret_access_key,
            s3_endpoint,
            s3_bucket,
            s3_region,
        )
        .with_auth(auth_method)
        .build()
        .unwrap();

    // scancel related signals
    #[cfg(feature = "job_cleanup")]
    let signals = Signals::new([SIGTERM, SIGCONT])?;
    #[cfg(feature = "job_cleanup")]
    let handle = signals.handle();

    let f = File::open(args.input).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let job: serde_json::Value = serde_json::from_str(contents.as_str()).unwrap();

    let primitive_job = client
        .run_primitive(
            &backend_name,
            program_id.parse().unwrap(),
            timeout_secs,
            log_level,
            &job,
            job_id,
        )
        .await?;

    #[cfg(feature = "job_cleanup")]
    let signals_task = tokio::spawn(handle_signals(signals, primitive_job.clone()));

    let mut succeeded: bool = true;
    match primitive_job.wait_for_final_state(None).await {
        Ok(retval) => match retval.status {
            JobStatus::Completed => {}
            JobStatus::Failed => {
                succeeded = false;
                if let Some(reason) = retval.reason_message {
                    println!("Job {} was failed. Reason {}", primitive_job.job_id, reason);
                } else {
                    println!("Job {} was failed.", primitive_job.job_id);
                }
            }
            JobStatus::Cancelled => {
                succeeded = false;
                println!("Job {} was cancelled.", primitive_job.job_id);
            }
            _ => unreachable!(),
        },
        Err(e) => {
            eprintln!(
                "Error occurred while waiting for final state: {:?}",
                e.to_string()
            );
            #[cfg(feature = "job_cleanup")]
            let _ = primitive_job.cancel(false).await;
            succeeded = false;
        }
    }

    if succeeded {
        match primitive_job.get_result::<serde_json::Value>().await {
            Ok(retval) => {
                let serialized = serde_json::to_string_pretty(&retval).unwrap();
                // output result to stdout, so that slurm copied to slurm-n.out file.
                if let Some(results_file) = args.results {
                    write_to_file(&results_file, serialized.as_bytes());
                } else {
                    println!("{}", serialized);
                }
            }
            Err(e) => {
                eprintln!(
                    "Error occurred while fetching job result from S3 bucket: {:?}",
                    e.to_string()
                );
            }
        }
    }

    match primitive_job.get_logs().await {
        Ok(retval) => {
            // It it enough to simply write to stdout so that Slurm will output to the
            // log file specified by sbatch --output option.
            println!("{}", retval);
        }
        Err(e) => {
            eprintln!(
                "Error occurred while fetching logs from S3 bucket: {:?}",
                e.to_string()
            );
        }
    }

    #[cfg(feature = "job_cleanup")]
    {
        client.delete_job(&primitive_job.job_id).await?;

        handle.close();
        signals_task.await?;
    }

    Ok(())
}
