#!/bin/bash
# Prolog: Set up IBM Quantum session if using QRMI_RESOURCE_ID as backend

echo "Setting up IBM Quantum session..."

source .env

export SESSION_INSTANCE="${SESSION_HUB}/${SESSION_GROUP}/${SESSION_PROJECT}"

# Ensure required secret and configuration variables are available.
# Report error and exit if any are missing.
if [ -z "$SESSION_INSTANCE" ]; then
  echo "Error: SESSION_INSTANCE is not set."
  exit 1
fi

if [ -z "$IAM_APIKEY" ]; then
  echo "Error: IAM_APIKEY is not set."
  exit 1
fi

if [ -z "$RESOURCE_ID" ]; then
  echo "Error: RESOURCE_ID is not set."
  exit 1
fi

# Set defaults for session TTL and mode if not provided.
export SESSION_MAX_TTL=${SESSION_MAX_TTL:-60}
export SESSION_MODE=${SESSION_MODE:-"dedicated"}

# Create a session via the IBM Quantum API
response=$(curl -X POST "https://api.quantum-computing.ibm.com/runtime/sessions" \
-H "Authorization: Bearer $IAM_APIKEY" \
-H "Accept: application/json" \
-H "Content-Type: application/json" \
-d '{
    "backend": "'"$RESOURCE_ID"'",
    "instance": "'"$SESSION_INSTANCE"'",
    "max_session_ttl": '"$SESSION_MAX_TTL"',
    "mode": "'"$SESSION_MODE"'"
}')

# Extract the session id using jq (ensure jq is installed)
SESSION_ID=$(echo "$response" | jq -r '.id')

if [ "$SESSION_ID" != "null" ] && [ -n "$SESSION_ID" ]; then
export SESSION_ID
echo "Session started with session_id: $SESSION_ID"
else
echo "Failed to start session. Response: $response"
exit 1
fi

