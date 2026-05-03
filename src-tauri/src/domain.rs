use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderId {
    Openai,
    Anthropic,
}

impl ProviderId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Openai => "openai",
            Self::Anthropic => "anthropic",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Openai => "OpenAI",
            Self::Anthropic => "Anthropic Claude",
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageSource {
    OfficialApi,
    ExperimentalLocalOauth,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAccountRecord {
    pub id: Option<i64>,
    pub provider_id: String,
    pub external_account_id: Option<String>,
    pub display_name: String,
    pub email: Option<String>,
    pub status: String,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceScopeRecord {
    pub id: Option<i64>,
    pub provider_account_id: i64,
    pub scope_type: String,
    pub external_scope_id: Option<String>,
    pub parent_scope_id: Option<i64>,
    pub display_name: String,
    pub is_default: bool,
    pub first_seen_at: String,
    pub last_seen_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSourceRecord {
    pub id: Option<i64>,
    pub provider_id: String,
    pub source_kind: String,
    pub credential_ref: Option<String>,
    pub source_label: Option<String>,
    pub is_enabled: bool,
    pub last_validated_at: Option<String>,
    pub last_error: Option<String>,
    pub last_success_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionStateRecord {
    pub id: Option<i64>,
    pub workspace_scope_id: i64,
    pub connection_source_id: i64,
    pub plan_code: Option<String>,
    pub plan_label: Option<String>,
    pub billing_status: String,
    pub usage_access: bool,
    pub detected_at: String,
    pub effective_from: Option<String>,
    pub effective_to: Option<String>,
    pub is_current: bool,
    pub confidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntitlementRecord {
    pub id: Option<i64>,
    pub subscription_state_id: i64,
    pub entitlement_kind: String,
    pub metric_key: Option<String>,
    pub used_value: Option<f64>,
    pub limit_value: Option<f64>,
    pub unit: Option<String>,
    pub window_seconds: Option<i64>,
    pub resets_at: Option<String>,
    pub raw_label: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageSnapshotV2Record {
    pub id: Option<i64>,
    pub workspace_scope_id: i64,
    pub connection_source_id: i64,
    pub usage_date: String,
    pub model_key: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub total_tokens: u64,
    pub request_count: u64,
    pub cost_usd: Option<f64>,
    pub quota_used: u64,
    pub quota_limit: Option<i64>,
    pub confidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunRecord {
    pub id: Option<i64>,
    pub connection_source_id: i64,
    pub provider_id: String,
    pub status: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub message: Option<String>,
    pub discovered_accounts: i32,
    pub discovered_scopes: i32,
    pub wrote_snapshots: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawObservationRecord {
    pub id: Option<i64>,
    pub connection_source_id: i64,
    pub observation_kind: String,
    pub captured_at: String,
    pub payload_json: String,
    pub redacted: bool,
}
