#!/bin/bash
# Epilog: Close the IBM Quantum session if one was established

source .env

if [ -n "$QRMI_IBM_QRS_SESSION_ID" ]; then
  echo "Closing IBM Quantum session with session_id: $QRMI_IBM_QRS_SESSION_ID"
  
  # Close the session via the DELETE endpoint
  response=$(curl -X DELETE \
  "https://quantum.cloud.ibm.com/api/v1/sessions/$QRMI_IBM_QRS_SESSION_ID/close" \
    -H "Authorization: Bearer $QRMI_IBM_DA_IAM_APIKEY" \
    -H "Accept: application/json")
  
  echo "Session closed. Response: $response"
  
  # Clean up environment variables
  unset QRMI_IBM_QRS_SESSION_ID
fi
