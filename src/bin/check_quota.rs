use anyhow::Result;
use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    #[serde(rename = "claudeAiOauth")]
    pub claude_ai_oauth: OAuthTokens,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OAuthTokens {
    pub access_token: String,
}

fn main() -> Result<()> {
    let output = Command::new("security")
        .args([
            "find-generic-password",
            "-s",
            "Claude Code-credentials",
            "-w",
        ])
        .output()?;
        
    let json_str = String::from_utf8(output.stdout)?;
    let creds: Credentials = serde_json::from_str(&json_str)?;

    let client = reqwest::blocking::Client::new();
    let response = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .header(
            "Authorization",
            format!("Bearer {}", creds.claude_ai_oauth.access_token),
        )
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Content-Type", "application/json")
        .send()?;

    println!("Status: {}", response.status());
    println!("Headers:");
    for (key, value) in response.headers() {
        println!("  {}: {:?}", key, value);
    }
    
    let text = response.text()?;
    println!("Body:\n{}", text);

    Ok(())
}
