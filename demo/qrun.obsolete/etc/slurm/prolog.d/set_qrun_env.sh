#!/bin/bash
# /etc/slurm/prolog.d/set_cloud_env.sh

if [ -n $SLURM_JOB_PARTITION ]
then
  if [ $SLURM_JOB_PARTITION == "staging" ]
  then
    echo "export IBMQRUN_AWS_ACCESS_KEY_ID=$(cat /etc/cloud_secrets/aws_access_key)"
    echo "export IBMQRUN_AWS_SECRET_ACCESS_KEY=$(cat /etc/cloud_secrets/aws_secret_access_key)"
    echo "export IBMQRUN_IAM_APIKEY=$(cat /etc/cloud_secrets/ibmcloud_iam_apikey)"
    echo "export IBMQRUN_SERVICE_CRN=$(cat /etc/cloud_secrets/directaccess_service_crn)"
    echo "export IBMQRUN_IAM_ENDPOINT=https://iam.test.cloud.ibm.com"
    echo "export IBMQRUN_DAAPI_ENDPOINT=http://127.0.0.1:8080"
    echo "export IBMQRUN_S3_ENDPOINT=https://s3.us-east.cloud-object-storage.appdomain.cloud"
    echo "export IBMQRUN_S3_BUCKET=<your bucket name>"
    echo "export IBMQRUN_S3_REGION=us-east"
  elif [ $SLURM_JOB_PARTITION == "normal" ]
  then
    echo "export IBMQRUN_AWS_ACCESS_KEY_ID=minioadmin"
    echo "export IBMQRUN_AWS_SECRET_ACCESS_KEY=minioadmin"
    echo "export IBMQRUN_IAM_APIKEY=demoapikey1"
    echo "export IBMQRUN_SERVICE_CRN=crn:v1:local:daa_sim"
    echo "export IBMQRUN_IAM_ENDPOINT=http://daapi:8290"
    echo "export IBMQRUN_DAAPI_ENDPOINT=http://daapi:8290"
    echo "export IBMQRUN_S3_ENDPOINT=http://minio:9000"
    echo "export IBMQRUN_S3_BUCKET=test"
    echo "export IBMQRUN_S3_REGION=us-east"
  fi
fi
