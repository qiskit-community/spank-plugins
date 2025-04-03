#!/bin/bash
# Prolog: Set up session if using QRMI_RESOURCE_ID as backend

echo "Setting up IBM Quantum session..."

# Load variables (from your .env file or Spank-plugin)
set -a        # Enable automatic export of all variables
source .env

# Ensure required configuration variables are available.
if [ -z "$QRMI_IBM_QRS_SERVICE_CRN" ]; then
  echo "Error: QRMI_IBM_QRS_SERVICE_CRN is not set."
fi

if [ -z "$QRMI_IBM_QRS_IAM_APIKEY" ]; then
  echo "Error: QRMI_IBM_QRS_IAM_APIKEY is not set."
fi

if [ -z "$QRMI_RESOURCE_ID" ]; then
  echo "Error: QRMI_RESOURCE_ID is not set."
fi

# Set defaults for session TTL and session mode if not provided.
export QRMI_IBM_QRS_SESSION_MAX_TTL=${QRMI_IBM_QRS_SESSION_MAX_TTL:-60}
export QRMI_IBM_QRS_SESSION_MODE=${QRMI_IBM_QRS_SESSION_MODE:-"dedicated"}

# For testing purposes, we invoke the acquire here.
echo "Invoking QRMI acquire function via inline Python..."

# Inline Python to read env variables and call acquire from qrmi
python3 <<'EOF'
import os
from qrmi import IBMQiskitRuntimeService

# The service constructor reads the required env variables.
service = IBMQiskitRuntimeService()

# Use QRMI_RESOURCE_ID (or another identifier if needed)
resource_id = os.getenv("QRMI_RESOURCE_ID", "default_resource")
try:
    session_id = service.acquire(resource_id)
    os.environ['QRMI_IBM_QRS_SESSION_ID'] = session_id
    print(session_id, end="")
except Exception as e:
    print(f"Error acquiring session: {e}", flush=True)
    exit(1)
EOF


# Check that a session ID was returned and export it.
if [ -z "$QRMI_IBM_QRS_SESSION_ID" ]; then
  echo "Failed to acquire session."
fi

export QRMI_IBM_QRS_SESSION_ID
set +a