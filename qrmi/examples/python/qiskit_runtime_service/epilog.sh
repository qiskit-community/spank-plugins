#!/bin/bash
# Epilog: Close the session if one was established
set -a        # Enable automatic export of all variables
source .env

if [ -n "$QRMI_IBM_QRS_SESSION_ID" ]; then
  echo "Closing IBM Quantum session with session_id: $QRMI_IBM_QRS_SESSION_ID"
  
# Inline Python to read env variables and call release from qrmi
python3 <<'EOF'
import os
from qrmi import IBMQiskitRuntimeService

# The service constructor reads the required env variables.
service = IBMQiskitRuntimeService()

# Use QRMI_IBM_QRS_SESSION_ID (or another identifier if needed)
session_id = os.getenv("QRMI_IBM_QRS_SESSION_ID", "default_resource")
try:
    service.release(session_id)
except Exception as e:
    print(f"Error releasing session: {e}", flush=True)
    exit(1)
EOF

  # Clean up environment variables
  unset QRMI_IBM_QRS_SESSION_ID
fi
set +a