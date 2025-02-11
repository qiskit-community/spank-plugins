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

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use retry_policies::policies::ExponentialBackoff;
use retry_policies::Jitter;
use std::env;
use std::time::Duration;

use direct_access_api::{AuthMethod, ClientBuilder};

#[tokio::main]
#[allow(unreachable_code)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Mismatched number of arguments. qrun <path to Qiskit PUBs JSON file>");
    }
    let job_file = args.get(1).unwrap();

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

    env_logger::init();

    let retry_policy = ExponentialBackoff::builder()
        .retry_bounds(Duration::from_secs(1), Duration::from_secs(5))
        .jitter(Jitter::Bounded)
        .base(2)
        .build_with_max_retries(5);

    let mut binding = ClientBuilder::new(daapi_endpoint);
    let mut base_builder = binding
        .with_timeout(Duration::from_secs(60))
        .with_retry_policy(retry_policy)
        .with_s3bucket(
            aws_access_key_id,
            aws_secret_access_key,
            s3_endpoint,
            s3_bucket,
            s3_region,
        );

    #[cfg(feature = "ibmcloud_appid_auth")]
    {
        // (Deprecated) AppId based authentication
        let appid_client_id = env::var("IBMQRUN_APPID_CLIENT_ID").expect("IBMQRUN_APPID_CLIENT_ID");
        let appid_secret = env::var("IBMQRUN_APPID_SECRET").expect("IBMQRUN_APPID_SECRET");

        base_builder = base_builder.with_auth(AuthMethod::IbmCloudAppId {
            username: appid_client_id,
            password: appid_secret,
        });
    }
    #[cfg(not(feature = "ibmcloud_appid_auth"))]
    {
        // IAM based authentication
        let iam_apikey = env::var("IBMQRUN_IAM_APIKEY").expect("IBMQRUN_IAM_APIKEY");
        let service_crn = env::var("IBMQRUN_SERVICE_CRN").expect("IBMQRUN_SERVICE_CRN");
        let iam_endpoint_url = env::var("IBMQRUN_IAM_ENDPOINT").expect("IBMQRUN_IAM_ENDPOINT");

        base_builder = base_builder.with_auth(AuthMethod::IbmCloudIam {
            apikey: iam_apikey,
            service_crn,
            iam_endpoint_url,
        });
    }

    let client = base_builder.build().unwrap();

    let f = File::open(job_file).expect("file not found");
    let mut buf_reader = BufReader::new(f);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    let job: serde_json::Value = serde_json::from_str(contents.as_str()).unwrap();

    let primitive_job = client
        .run_primitive(
            &backend_name,
            program_id.parse().unwrap(),
            86400,
            "debug".parse().unwrap(),
            &job,
        )
        .await?;
    match primitive_job.wait_for_final_state(Some(1800.0)).await {
        Ok(_retval) => {
            //println!("{}", serde_json::to_string_pretty(&retval).unwrap());
        }
        Err(e) => {
            println!(
                "Error occurred while waiting for final state: {:?}",
                e.to_string()
            );
            primitive_job.cancel(false).await?;
        }
    }

    match primitive_job.get_result::<serde_json::Value>().await {
        Ok(retval) => {
            println!("{}", serde_json::to_string_pretty(&retval).unwrap());
        }
        Err(e) => {
            println!(
                "Error occurred while fetching job result from S3 bucket: {:?}",
                e.to_string()
            );
        }
    }

    client.delete_job(&primitive_job.job_id).await?;

    Ok(())
}
