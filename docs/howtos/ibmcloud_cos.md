# Using IBM Cloud COS as S3 compatible storage

This document describes how to use IBM Cloud COS as S3-compatible storage, specifically, how to obtain the AWS Access Key ID (`QRMI_IBM_DA_AWS_ACCESS_KEY_ID`), the AWS Secret Access Key (`QRMI_IBM_DA_AWS_SECRET_ACCESS_KEY`), and the S3 endpoint URL (`QRMI_IBM_DA_S3_ENDPOINT`).

## Prerequisite

IBM Cloud Object Storage instance and bucket -- Go to the [IBM Cloud Object Storage web page](https://cloud.ibm.com/objectstorage/overview) to create an S3 instance and a bucket in your instance.

## How to obtain the AWS Access Key ID and Secret Access Key

To create your credentials, navigate to the `Service credentials` tab in your instance's web page. All instances can be found in the [IBM Cloud Instances web page](https://cloud.ibm.com/objectstorage/instances). Click on `New Credential` in the `Service credential` tab to create your HMAC (Hash-based Message Authentication Code) credentials.

HMAC credentials consist of an Access Key and Secret Key paired for use with S3-compatible tools and libraries that require authentication. Users can create a set of HMAC credentials as part of a Service Credential by switching the `Include HMAC Credential` to `On` as shown below:

![include_HMAC_credential](https://cloud.ibm.com/docs-content/v4/content/3842758572478f973a02d6e5afad955eb1a777d2/cloud-object-storage/images/hmac-credential-dialog.jpg)

After the Service Credential is created, the HMAC credentials are included in the `cos_hmac_keys` field as shown below. Click on the `v` on the left to expose the full Service Credential. `access_key_id` is AWS Access Key ID and `secret_access_key` is AWS Secret Access Key. 

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

You can also use the IBM Cloud CLI to create the credentials as shown below. The `access_key_id` and `secret_access_key` will be output from the command.

```bash
ibmcloud resource service-key-create <key-name-without-spaces> Writer --instance-name "<instance name--use quotes if your instance name has spaces>" --parameters '{"HMAC":true}'
```

Refer to the [IBM Cloud documentation](https://cloud.ibm.com/docs/cloud-object-storage?topic=cloud-object-storage-uhc-hmac-credentials-main) for details about using HMAC credentials.

## How to obtain the S3 endpoint URL

S3 endpoints can be found in the [IBM Cloud Object Storage Regional Endpoints list](https://cloud.ibm.com/docs/cloud-object-storage?topic=cloud-object-storage-endpoints#endpoints-region). Choose one to fit to your IBM Cloud Object Storage instance. For example, if your instance is located in the `us-east` region then the endpoint for your instance is `https://s3.us-east.cloud-object-storage.appdomain.cloud`.


## END OF DOCUMENT
