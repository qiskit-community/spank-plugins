#!/bin/bash
# Prolog: Set up IBM Quantum session if using QRMI_RESOURCE_ID as backend

echo "Setting up IBM Quantum session..."

# Load variables, this should come from Spank-plugin
#----------------------------------------------------
source .env
#----------------------------------------------------

echo "Setting up IBM Quantum session..."

# Ensure required secret and configuration variables are available.
# Report error and exit if any are missing.
if [ -z "$QRMI_IBM_DA_SERVICE_CRN" ]; then
  echo "Error: QRMI_IBM_DA_SERVICE_CRN is not set."
  exit 1
fi

if [ -z "$QRMI_IBM_DA_IAM_APIKEY" ]; then
  echo "Error: QRMI_IBM_DA_IAM_APIKEY is not set."
  exit 1
fi

if [ -z "$QRMI_RESOURCE_ID" ]; then
  echo "Error: QRMI_RESOURCE_ID is not set."
  exit 1
fi

# Set defaults for session TTL and mode if not provided.
SESSION_MAX_TTL=${SESSION_MAX_TTL:-60}
SESSION_MODE=${SESSION_MODE:-"dedicated"}

# Create a session via the IBM Quantum API
response=$(curl -X POST "https://quantum.cloud.ibm.com/api/v1/sessions" \
-H "Accept: application/json" \
-H "Authorization: Bearer $QRMI_IBM_DA_IAM_APIKEY" \
-H "Service-CRN: $QRMI_IBM_DA_SERVICE_CRN" \
-H "Content-Type: application/json" \
  --data-raw "{
  'mode': $SESSION_MODE,
  'max_ttl': $SESSION_MAX_TTL
  }"
  )

# Extract the session id using jq (ensure jq is installed)
QRMI_IBM_QRS_SESSION_ID=$(echo "$response" | jq -r '.id')

if [ "$QRMI_IBM_QRS_SESSION_ID" != "null" ] && [ -n "$QRMI_IBM_QRS_SESSION_ID" ]; then
export QRMI_IBM_QRS_SESSION_ID
echo "Session started with session_id: $QRMI_IBM_QRS_SESSION_ID"
else
echo "Failed to start session. Response: $response"
exit 1
fi

