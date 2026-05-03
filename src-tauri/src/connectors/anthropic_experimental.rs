use crate::connectors::{
    AccountHint, ConnectionSourceKind, ConnectorError, DetectedAccount, DetectedEntitlement,
    DetectedRawObservation, DetectedScope, DetectedSubscription, ProviderConnector, SyncPackage,
    ValidationResult,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const PROD_REFRESH_URL: &str = "https://platform.claude.com/v1/oauth/token";
const PROD_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";
const SCOPES: &str = "user:profile user:inference user:sessions:claude_code user:mcp_servers user:file_upload";
const REFRESH_BUFFER_MS: i64 = 5 * 60 * 1000;
const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

fn default_claude_home() -> PathBuf {
    dirs::home_dir()
        .map(|p| p.join(".claude"))
        .unwrap_or_else(|| PathBuf::from("~/.claude"))
}

fn claude_credentials_path() -> PathBuf {
    std::env::var("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| default_claude_home())
        .join(".credentials.json")
}

fn try_parse_json<T: for<'a> Deserialize<'a>>(text: &str) -> Option<T> {
    serde_json::from_str(text).ok()
}

fn try_parse_hex_utf8(text: &str) -> Option<String> {
    let hex = text.trim().trim_start_matches("0x").trim_start_matches("0X");
    if hex.is_empty() || hex.len() % 2 != 0 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let chars: Vec<char> = hex.chars().collect();
    let mut bytes = Vec::with_capacity(chars.len() / 2);
    for pair in chars.chunks(2) {
        if pair.len() < 2 {
            break;
        }
        let a = pair[0].to_digit(16)? as u8;
        let b = pair[1].to_digit(16)? as u8;
        bytes.push((a << 4) | b);
    }
    String::from_utf8(bytes).ok()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CredentialsFile {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<OAuthCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthCredentials {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "refreshToken")]
    refresh_token: Option<String>,
    #[serde(rename = "expiresAt")]
    expires_at: Option<i64>,
    scopes: Option<Vec<String>>,
    #[serde(rename = "subscriptionType")]
    subscription_type: Option<String>,
    #[serde(rename = "rateLimitTier")]
    rate_limit_tier: Option<String>,
}

fn load_credentials_from_file(path: &PathBuf) -> Option<OAuthCredentials> {
    let text = std::fs::read_to_string(path).ok()?;
    let parsed: CredentialsFile = try_parse_json(&text)?;
    parsed.claude_ai_oauth.filter(|c| !c.access_token.is_empty())
}

fn load_credentials_from_keychain() -> Option<OAuthCredentials> {
    let suffix = std::env::var("CLAUDE_CONFIG_DIR")
        .map(|d| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            d.hash(&mut hasher);
            format!("-{:x}", hasher.finish())[..8].to_string()
        })
        .unwrap_or_default();
    let service = format!("Claude Code{}-credentials", suffix);
    let entry = keyring::Entry::new(&service, "claude").ok()?;
    let text = entry.get_password().ok()?;
    try_parse_json::<CredentialsFile>(&text)
        .and_then(|f| f.claude_ai_oauth)
        .or_else(|| {
            try_parse_hex_utf8(&text)
                .and_then(|t| try_parse_json::<CredentialsFile>(&t))
                .and_then(|f| f.claude_ai_oauth)
        })
}

fn load_credentials() -> Option<OAuthCredentials> {
    load_credentials_from_file(&claude_credentials_path())
        .or_else(load_credentials_from_keychain)
}

fn needs_refresh(creds: &OAuthCredentials) -> bool {
    let Some(expires_at) = creds.expires_at else {
        return true;
    };
    let now_ms = chrono::Utc::now().timestamp_millis();
    expires_at - REFRESH_BUFFER_MS < now_ms
}

fn save_credentials(creds: &OAuthCredentials) -> Result<(), String> {
    let path = claude_credentials_path();
    let creds_file = CredentialsFile {
        claude_ai_oauth: Some(creds.clone()),
    };
    let text = serde_json::to_string(&creds_file).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&path, text).map_err(|e| e.to_string())
}

fn format_plan_label(subscription_type: &Option<String>, rate_limit_tier: &Option<String>) -> String {
    let base = subscription_type
        .as_ref()
        .map(|s| {
            let mut c = s.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .unwrap_or_else(|| "Unknown".to_string());
    if let Some(tier) = rate_limit_tier.as_ref() {
        if let Some(m) = tier.strip_suffix('x') {
            return format!("{} {}x", base, m);
        }
    }
    base
}

async fn do_refresh_token(creds: &OAuthCredentials) -> Result<String, ConnectorError> {
    let refresh_tok = creds
        .refresh_token
        .clone()
        .ok_or_else(|| ConnectorError::RefreshFailed("No refresh token".to_string()))?;
    let body = serde_json::json!({
        "grant_type": "refresh_token",
        "refresh_token": refresh_tok,
        "client_id": PROD_CLIENT_ID,
        "scope": SCOPES,
    });
    let client = Client::new();
    let resp = client
        .post(PROD_REFRESH_URL)
        .header("Content-Type", "application/json")
        .json(&body)
        .timeout(std::time::Duration::from_secs(15))
        .send()
        .await
        .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;
    let status = resp.status();
    if status == 400 || status == 401 {
        let body_text = resp.text().await.unwrap_or_default();
        if body_text.contains("invalid_grant") {
            return Err(ConnectorError::RefreshFailed(
                "Session expired. Run `claude` to log in again.".to_string(),
            ));
        }
        return Err(ConnectorError::RefreshFailed(format!(
            "Token refresh failed: {}",
            status
        )));
    }
    if !status.is_success() {
        return Err(ConnectorError::RefreshFailed(format!(
            "Unexpected status: {}",
            status
        )));
    }
    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| ConnectorError::ParseError(e.to_string()))?;
    let new_access = json
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConnectorError::ParseError("Missing access_token".to_string()))?
        .to_string();
    let mut updated = creds.clone();
    updated.access_token = new_access.clone();
    if let Some(new_refresh) = json.get("refresh_token").and_then(|v| v.as_str()) {
        updated.refresh_token = Some(new_refresh.to_string());
    }
    if let Some(expires_in) = json.get("expires_in").and_then(|v| v.as_i64()) {
        updated.expires_at = Some(chrono::Utc::now().timestamp_millis() + expires_in * 1000);
    }
    let _ = save_credentials(&updated);
    Ok(new_access)
}

async fn do_fetch_usage(access_token: &str) -> Result<serde_json::Value, ConnectorError> {
    let client = Client::new();
    let resp = client
        .get(USAGE_URL)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", "claude-code/2.1.69")
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| ConnectorError::NetworkError(e.to_string()))?;
    if resp.status() == 401 {
        return Err(ConnectorError::InvalidCredential(
            "Token expired. Run `claude` to log in again.".to_string(),
        ));
    }
    if resp.status() == 429 {
        let retry_after = resp
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        return Err(ConnectorError::RateLimited {
            retry_after_secs: retry_after,
        });
    }
    if !resp.status().is_success() {
        return Err(ConnectorError::UsageFetchFailed(format!(
            "HTTP {}",
            resp.status()
        )));
    }
    resp.json()
        .await
        .map_err(|e| ConnectorError::ParseError(e.to_string()))
}

fn parse_entitlement_window(
    obj: &serde_json::Map<String, serde_json::Value>,
    kind: &str,
    label: &str,
) -> Option<DetectedEntitlement> {
    let utilization = obj.get("utilization")?.as_f64()?;
    let resets_at = obj.get("resets_at").and_then(|v| v.as_str()).map(|s| s.to_string());
    Some(DetectedEntitlement {
        subscription_state_id: 0,
        entitlement_kind: kind.to_string(),
        metric_key: Some(label.to_string()),
        used_value: Some(utilization),
        limit_value: Some(100.0),
        unit: Some("percent".to_string()),
        window_seconds: None,
        resets_at,
        raw_label: Some(label.to_string()),
    })
}

pub struct AnthropicExperimentalConnector {
    _http_client: Client,
}

impl AnthropicExperimentalConnector {
    pub fn new(http_client: Client) -> Self {
        Self {
            _http_client: http_client,
        }
    }

    fn source_kind() -> ConnectionSourceKind {
        ConnectionSourceKind::ExperimentalLocalOauth
    }

    async fn do_sync(&self, creds: &OAuthCredentials) -> Result<SyncPackage, ConnectorError> {
        let mut access_token = creds.access_token.clone();

        if needs_refresh(creds) {
            if let Ok(new_token) = do_refresh_token(creds).await {
                access_token = new_token;
            }
        }

        let usage_data = do_fetch_usage(&access_token).await?;

        let detected_at = chrono::Utc::now().to_rfc3339();
        let first_seen = detected_at.clone();
        let last_seen = detected_at.clone();

        let plan_label = format_plan_label(&creds.subscription_type, &creds.rate_limit_tier);

        let mut entitlements = Vec::new();

        if let Some(obj) = usage_data.get("five_hour").and_then(|v| v.as_object()) {
            if let Some(ent) = parse_entitlement_window(obj, "token_window", "5h") {
                entitlements.push(ent);
            }
        }
        if let Some(obj) = usage_data.get("seven_day").and_then(|v| v.as_object()) {
            if let Some(ent) = parse_entitlement_window(obj, "weekly_quota", "7d") {
                entitlements.push(ent);
            }
        }
        if let Some(obj) = usage_data.get("seven_day_sonnet").and_then(|v| v.as_object()) {
            if let Some(ent) = parse_entitlement_window(obj, "model_limit", "sonnet") {
                entitlements.push(ent);
            }
        }
        if let Some(obj) = usage_data.get("seven_day_omelette").and_then(|v| v.as_object()) {
            if let Some(ent) = parse_entitlement_window(obj, "model_limit", "claude_design") {
                entitlements.push(ent);
            }
        }
        if let Some(obj) = usage_data.get("extra_usage").and_then(|v| v.as_object()) {
            if let (Some(used), Some(limit)) = (
                obj.get("used_credits").and_then(|v| v.as_i64()),
                obj.get("monthly_limit").and_then(|v| v.as_i64()),
            ) {
                entitlements.push(DetectedEntitlement {
                    subscription_state_id: 0,
                    entitlement_kind: "credits_balance".to_string(),
                    metric_key: Some("extra".to_string()),
                    used_value: Some(used as f64 / 100.0),
                    limit_value: Some(limit as f64 / 100.0),
                    unit: Some("dollars".to_string()),
                    window_seconds: None,
                    resets_at: None,
                    raw_label: Some("Extra Usage".to_string()),
                });
            }
        }

        let scopes = vec![DetectedScope {
            provider_account_id: 0,
            scope_type: "personal".to_string(),
            external_scope_id: None,
            parent_scope_id: None,
            display_name: "Personal".to_string(),
            is_default: true,
            first_seen_at: first_seen.clone(),
            last_seen_at: last_seen.clone(),
        }];

        let subscriptions = vec![DetectedSubscription {
            workspace_scope_id: 0,
            connection_source_id: 0,
            plan_code: creds.subscription_type.clone(),
            plan_label: Some(plan_label),
            billing_status: "active".to_string(),
            usage_access: true,
            detected_at: detected_at.clone(),
            effective_from: None,
            effective_to: None,
            is_current: true,
            confidence: "medium".to_string(),
        }];

        Ok(SyncPackage {
            accounts: vec![DetectedAccount {
                provider_id: "anthropic".to_string(),
                external_account_id: None,
                display_name: "Claude Code Account".to_string(),
                email: None,
                status: "active".to_string(),
                first_seen_at: first_seen.clone(),
                last_seen_at: last_seen.clone(),
            }],
            scopes,
            subscriptions,
            entitlements,
            usage_snapshots: Vec::new(),
            raw_observations: vec![DetectedRawObservation {
                connection_source_id: 0,
                observation_kind: "usage_response".to_string(),
                captured_at: detected_at,
                payload_json: serde_json::to_string(&usage_data).unwrap_or_default(),
                redacted: true,
            }],
        })
    }
}

#[async_trait]
impl ProviderConnector for AnthropicExperimentalConnector {
    fn provider_id(&self) -> &'static str {
        "anthropic"
    }

    fn source_kind(&self) -> ConnectionSourceKind {
        Self::source_kind()
    }

    async fn validate(
        &self,
        _credential_ref: &str,
    ) -> Result<ValidationResult, ConnectorError> {
        let creds = load_credentials().ok_or_else(|| {
            ConnectorError::InvalidCredential("No Claude Code credentials found".to_string())
        })?;
        let can_use = !creds.access_token.is_empty();
        Ok(ValidationResult {
            valid: true,
            can_access_usage: can_use,
            can_access_subscription: can_use,
            account_hint: Some(AccountHint {
                display_name: Some("Claude Code Account".to_string()),
                email: None,
            }),
        })
    }

    async fn sync(&self, _credential_ref: &str) -> Result<SyncPackage, ConnectorError> {
        let creds = load_credentials().ok_or_else(|| {
            ConnectorError::InvalidCredential("No Claude Code credentials found".to_string())
        })?;
        self.do_sync(&creds).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_parse_hex_utf8_valid() {
        assert_eq!(try_parse_hex_utf8("48656c6c6f"), Some("Hello".to_string()));
        assert_eq!(try_parse_hex_utf8("0x48656c6c6f"), Some("Hello".to_string()));
        assert_eq!(try_parse_hex_utf8("0X48656c6c6f"), Some("Hello".to_string()));
    }

    #[test]
    fn test_try_parse_hex_utf8_empty() {
        assert_eq!(try_parse_hex_utf8(""), None);
    }

    #[test]
    fn test_try_parse_hex_utf8_invalid() {
        assert_eq!(try_parse_hex_utf8("G"), None);
        assert_eq!(try_parse_hex_utf8("12345"), None);
    }

    #[test]
    fn test_oauth_credentials_deserialization() {
        let json = r#"{"accessToken":"tok123","refreshToken":"ref456","expiresAt":1234567890,"scopes":["a","b"]}"#;
        let creds: OAuthCredentials = serde_json::from_str(json).unwrap();
        assert_eq!(creds.access_token, "tok123");
        assert_eq!(creds.refresh_token, Some("ref456".to_string()));
        assert_eq!(creds.expires_at, Some(1234567890));
        assert_eq!(creds.scopes, Some(vec!["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn test_credentials_file_deserialization() {
        let json = r#"{"claudeAiOauth":{"accessToken":"tok123","refreshToken":"ref456"}}"#;
        let file: CredentialsFile = serde_json::from_str(json).unwrap();
        assert!(file.claude_ai_oauth.is_some());
        assert_eq!(file.claude_ai_oauth.unwrap().access_token, "tok123");
    }

    #[test]
    fn test_credentials_file_missing_oauth() {
        let json = r#"{"someOther":"data"}"#;
        let file: CredentialsFile = serde_json::from_str(json).unwrap();
        assert!(file.claude_ai_oauth.is_none());
    }
}