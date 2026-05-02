use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Openai,
    Anthropic,
    Gemini,
    GithubCopilot,
    Opencode,
    Kimi,
    Minimax,
    Glm,
    Cursor,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::GithubCopilot => "github_copilot",
            Self::Opencode => "opencode",
            Self::Kimi => "kimi",
            Self::Minimax => "minimax",
            Self::Glm => "glm",
            Self::Cursor => "cursor",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Openai => "OpenAI",
            Self::Anthropic => "Anthropic Claude",
            Self::Gemini => "Google Gemini",
            Self::GithubCopilot => "GitHub Copilot",
            Self::Opencode => "Opencode",
            Self::Kimi => "Kimi",
            Self::Minimax => "Minimax",
            Self::Glm => "GLM",
            Self::Cursor => "Cursor",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageSource {
    OfficialApi,
    LocalEstimate,
    Manual,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    High,
    Medium,
    Low,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderStatus {
    Connected,
    NeedsCredentials,
    Experimental,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncEventStatus {
    Success,
    Warning,
    Error,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderCapabilities {
    pub tokens: bool,
    pub cost: bool,
    pub quota: bool,
    pub realtime: bool,
    pub historical: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSummary {
    pub id: ProviderId,
    pub name: String,
    pub status: ProviderStatus,
    pub source: UsageSource,
    pub confidence: Confidence,
    pub capabilities: ProviderCapabilities,
    pub daily_tokens: u64,
    pub weekly_tokens: u64,
    pub cost_usd: Option<f64>,
    pub quota_used: Option<u8>,
    pub quota_limit: Option<u64>,
    pub last_sync: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsagePoint {
    pub day: String,
    pub tokens: u64,
    pub cost_usd: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncEvent {
    pub provider_id: ProviderId,
    pub provider_name: String,
    pub status: SyncEventStatus,
    pub message: String,
    pub at: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSnapshot {
    pub providers: Vec<ProviderSummary>,
    pub history: Vec<UsagePoint>,
    pub sync_events: Vec<SyncEvent>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenAiConnectionState {
    pub has_credentials: bool,
    pub account_label: Option<String>,
    pub organization_id: Option<String>,
    pub project_id: Option<String>,
    pub last_validated_at: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub usage_access: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOpenAiCredentialsInput {
    pub api_key: String,
    pub account_label: Option<String>,
    pub organization_id: Option<String>,
    pub project_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveOpenAiCredentialsResult {
    pub connection: OpenAiConnectionState,
    pub message: String,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnthropicConnectionState {
    pub has_credentials: bool,
    pub account_label: Option<String>,
    pub last_validated_at: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub usage_access: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiConnectionState {
    pub has_credentials: bool,
    pub account_label: Option<String>,
    pub last_validated_at: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub usage_access: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGeminiCredentialsInput {
    pub api_key: String,
    pub account_label: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveGeminiCredentialsResult {
    pub connection: GeminiConnectionState,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAnthropicCredentialsInput {
    pub api_key: String,
    pub account_label: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveAnthropicCredentialsResult {
    pub connection: AnthropicConnectionState,
    pub message: String,
}
