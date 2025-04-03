#[derive(Debug)]
pub enum MyError {
    Reqwest(reqwest::Error),
    MissingToken,
}

impl From<reqwest::Error> for MyError {
    fn from(err: reqwest::Error) -> MyError {
        MyError::Reqwest(err)
    }
}

pub async fn fetch_access_token(api_key: &str) -> Result<String, MyError> {
    let client = reqwest::Client::new();
    let params = [
        ("grant_type", "urn:ibm:params:oauth:grant-type:apikey"),
        ("apikey", api_key),
    ];

    let response = client
        .post("https://iam.cloud.ibm.com/identity/token")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await?;
        
    let json: serde_json::Value = response.json().await?;
    
    if let Some(token) = json.get("access_token").and_then(serde_json::Value::as_str) {
        Ok(token.to_string())
    } else {
        Err(MyError::MissingToken)
    }
}
