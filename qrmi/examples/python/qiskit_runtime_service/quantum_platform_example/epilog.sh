#!/bin/bash
# Epilog: Close the IBM Quantum session if one was established

source .env

if [ -n "$SESSION_ID" ]; then
  echo "Closing IBM Quantum session with session_id: $SESSION_ID"
  
  # Close the session via the DELETE endpoint
  response=$(curl -s -X DELETE "https://api.quantum-computing.ibm.com/runtime/sessions/$SESSION_ID/close" \
    -H "Authorization: Bearer $IAM_APIKEY" \
    -H "Accept: application/json")
  
  echo "Session closed. Response: $response"
  
  # Clean up environment variables
  unset SESSION_ID
  unset QRMI_IAM_APIKEY
fi
