use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaygroundRequest {
    channel: String,
    mode: String,
    edition: String,
    #[serde(rename = "crateType")]
    crate_type: String,
    tests: bool,
    code: String,
    backtrace: bool,
}

#[derive(Debug, Deserialize)]
pub struct PlaygroundResponse {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub async fn execute_rust_code(code: String) -> Result<PlaygroundResponse, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let request = PlaygroundRequest {
        channel: "stable".to_string(),
        mode: "debug".to_string(),
        edition: "2021".to_string(),
        crate_type: "bin".to_string(),
        tests: false,
        code,
        backtrace: false,
    };

    let response = client
        .post("https://play.rust-lang.org/execute")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Playground API error: {}", response.status()));
    }

    let result = response
        .json::<PlaygroundResponse>()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(result)
}