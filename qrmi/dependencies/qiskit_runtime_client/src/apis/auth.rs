#[derive(Debug)]
pub enum AuthError {
    Reqwest(reqwest::Error),
    MissingToken,
}

impl From<reqwest::Error> for AuthError {
    fn from(err: reqwest::Error) -> AuthError {
        AuthError::Reqwest(err)
    }
}

/// Returns a bearer token based on api_key and iam_endpoint.
pub async fn fetch_access_token(api_key: &str, iam_endpoint: &str) -> Result<String, AuthError> {
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
        
    let json: serde_json::Value = response.json().await?;
    
    if let Some(token) = json.get("access_token").and_then(serde_json::Value::as_str) {
        Ok(token.to_string())
    } else {
        Err(AuthError::MissingToken)
    }
}
