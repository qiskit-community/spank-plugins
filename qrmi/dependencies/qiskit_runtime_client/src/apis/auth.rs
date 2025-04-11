use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum AuthError {
    Reqwest(reqwest::Error),
    AuthTokenError,
    InvalidResponse(String),
}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> AuthError {
        AuthError::Reqwest(err)
    }
}

/// Returns the current Unix timestamp in seconds.
fn current_unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

/// Returns a bearer token along with its expiration timestamp and computed lifetime (in seconds).
///
/// The JSON response is expected to include:
///   "access_token": "ACCESS_TOKEN",
///   "expiration": 1616750582,
///   "expires_in": 3600
///
/// If the response indicates an error via its HTTP status, the error response is built using the status code.
/// If the JSON contains error fields (e.g., "error" and "error_description"), they are also used in the message.
pub async fn fetch_access_token(
    api_key: &str,
    iam_endpoint: &str,
) -> Result<(String, i64, i64), AuthError> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "urn:ibm:params:oauth:grant-type:apikey"),
        ("apikey", api_key),
    ];

    let response = client
        .post(format!("{}/identity/token", iam_endpoint))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;

    let status = response.status();
    if !status.is_success() {
        // Attempt to get error details from the response body.
        let body = response.text().await.unwrap_or_default();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            if let (Some(error_code), Some(error_desc)) = (
                json.get("error").and_then(|v| v.as_str()),
                json.get("error_description").and_then(|v| v.as_str()),
            ) {
                return Err(AuthError::InvalidResponse(format!(
                    "HTTP {}: {}: {}",
                    status, error_code, error_desc
                )));
            }
        }
        return Err(AuthError::InvalidResponse(format!(
            "HTTP {}: {}",
            status, body
        )));
    }

    let json: serde_json::Value = response.json().await?;

    if let (Some(token), Some(expiration), Some(lifetime)) = (
        json.get("access_token").and_then(|v| v.as_str()),
        json.get("expiration").and_then(|v| v.as_i64()),
        json.get("expires_in").and_then(|v| v.as_i64()),
    ) {
        Ok((token.to_string(), expiration, lifetime))
    } else {
        Err(AuthError::AuthTokenError)
    }
}

/// Checks whether the current token is valid based on its remaining lifetime.
///
/// It uses two conditions:
/// 1. The remaining lifetime is less than 360 seconds.
/// 2. The remaining lifetime is less than 10% of the token's computed lifetime.
///
/// If either condition is met, it fetches a new token and updates the provided references.
pub async fn check_token(
    api_key: &str,
    iam_endpoint: &str,
    current_token: &mut Option<String>,
    token_expiration: &mut i64,
    token_lifetime: &mut i64,
) -> Result<(), AuthError> {
    let now = current_unix_timestamp();
    let remaining = *token_expiration - now;

    if remaining < 360 || remaining < (*token_lifetime / 10) {
        let (new_token, new_expiration, new_lifetime) =
            fetch_access_token(api_key, iam_endpoint).await?;
        *current_token = Some(new_token);
        *token_expiration = new_expiration;
        *token_lifetime = new_lifetime;
    }
    Ok(())
}
