# Using IBM Cloud COS as S3 compatible storage

This document describes how to use IBM Cloud COS as S3-compatible storage, specifically how to obtain the AWS Access Key ID and Secret Access Key for use with S3-compatible tools and libraries.

## Prerequisites
* IBM Cloud COS instance

## How to obtain AWS Access Key ID and Secret Access Key

Refer [this guide](https://cloud.ibm.com/docs/cloud-object-storage?topic=cloud-object-storage-uhc-hmac-credentials-main&locale=en) to obtain AWS Access Key ID and Secret Access Key.

`IBM Cloud -> Infrastructure -> Storage -> Objective Storage` in order to navigate IBM Cloud website. 

Then, `Create Instance` and `Create Bucket` in your instance, accordingly. After the bucket is created, navigate to your instance, and click `Service credentials` and Click `New Credentials` to create your credential with HMAC. 

HMAC credentials consist of an Access Key and Secret Key paired for use with S3-compatible tools and libraries that require authentication. Users can create a set of HMAC credentials as part of a Service Credential by switching the `Include HMAC Credential` to `On` during credential creation in the console. 

![include_HMAC_credential](https://cloud.ibm.com/docs-content/v4/content/3842758572478f973a02d6e5afad955eb1a777d2/cloud-object-storage/images/hmac-credential-dialog.jpg)

After the Service Credential is created, the HMAC Key is included in the `cos_hmac_keys` field like below. `access_key_id` is AWS Access Key ID and `secret_access_key` is AWS Secret Access Key. 

```bash
{
    "apikey": "P2Eh1K7CBjSOvhBEdhwxtNhJ****************",
    "cos_hmac_keys": {
        "access_key_id": "682c65a7643e460e***************",
        "secret_access_key": "******************8eb860c3e321af9e2f16"
    },
    "endpoints": "https://control.cloud-object-storage.cloud.ibm.com/v2/endpoints",
    "iam_apikey_description": ...
```

## How to obtain S3 endpoint URL

Service credential contains `endpoints` field. Open this URL and choose one to fit to your IBM Cloud COS instance. For example, if your instance is located in us-east region, `https://s3.us-east.cloud-object-storage.appdomain.cloud` is an endpoint for your instance.


END OF DOCUMENT
