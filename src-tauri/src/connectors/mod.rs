#![allow(dead_code)]

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionSourceKind {
    OfficialApi,
    ExperimentalLocalOauth,
    ExperimentalLocalCli,
    Manual,
}

impl ConnectionSourceKind {
    pub fn precedence(&self) -> u8 {
        match self {
            Self::OfficialApi => 0,
            Self::ExperimentalLocalOauth => 1,
            Self::ExperimentalLocalCli => 2,
            Self::Manual => 3,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountHint {
    pub display_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    pub can_access_usage: bool,
    pub can_access_subscription: bool,
    pub account_hint: Option<AccountHint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedAccount {
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
pub struct DetectedScope {
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
pub struct DetectedSubscription {
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
pub struct DetectedEntitlement {
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
pub struct DetectedUsageSnapshot {
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
pub struct DetectedRawObservation {
    pub connection_source_id: i64,
    pub observation_kind: String,
    pub captured_at: String,
    pub payload_json: String,
    pub redacted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncPackage {
    pub accounts: Vec<DetectedAccount>,
    pub scopes: Vec<DetectedScope>,
    pub subscriptions: Vec<DetectedSubscription>,
    pub entitlements: Vec<DetectedEntitlement>,
    pub usage_snapshots: Vec<DetectedUsageSnapshot>,
    pub raw_observations: Vec<DetectedRawObservation>,
}

impl SyncPackage {
    pub fn empty() -> Self {
        Self {
            accounts: Vec::new(),
            scopes: Vec::new(),
            subscriptions: Vec::new(),
            entitlements: Vec::new(),
            usage_snapshots: Vec::new(),
            raw_observations: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ConnectorError {
    InvalidCredential(String),
    RefreshFailed(String),
    UsageFetchFailed(String),
    RateLimited { retry_after_secs: Option<u64> },
    ParseError(String),
    NetworkError(String),
}

impl std::fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCredential(msg) => write!(f, "Invalid credential: {}", msg),
            Self::RefreshFailed(msg) => write!(f, "Token refresh failed: {}", msg),
            Self::UsageFetchFailed(msg) => write!(f, "Usage fetch failed: {}", msg),
            Self::RateLimited { retry_after_secs } => {
                if let Some(secs) = retry_after_secs {
                    write!(f, "Rate limited, retry after {} seconds", secs)
                } else {
                    write!(f, "Rate limited")
                }
            }
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for ConnectorError {}

#[async_trait]
pub trait ProviderConnector: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn source_kind(&self) -> ConnectionSourceKind;
    async fn validate(
        &self,
        credential_ref: &str,
    ) -> Result<ValidationResult, ConnectorError>;
    async fn sync(
        &self,
        credential_ref: &str,
    ) -> Result<SyncPackage, ConnectorError>;
}

#[derive(Default)]
pub struct ConnectorRegistry {
    connectors: Vec<Box<dyn ProviderConnector>>,
}

impl ConnectorRegistry {
    pub fn new() -> Self {
        Self {
            connectors: Vec::new(),
        }
    }

    pub fn register<C: ProviderConnector + 'static>(&mut self, connector: C) {
        self.connectors.push(Box::new(connector));
    }

    pub fn by_provider(&self, provider_id: &str) -> Vec<&dyn ProviderConnector> {
        self.connectors
            .iter()
            .filter(|c| c.provider_id() == provider_id)
            .map(|c| c.as_ref())
            .collect()
    }

    pub fn by_provider_and_source(
        &self,
        provider_id: &str,
        source_kind: &ConnectionSourceKind,
    ) -> Option<&dyn ProviderConnector> {
        self.connectors
            .iter()
            .filter(|c| c.provider_id() == provider_id && c.source_kind() == *source_kind)
            .map(|c| c.as_ref())
            .next()
    }

    pub fn all(&self) -> Vec<&dyn ProviderConnector> {
        self.connectors.iter().map(|c| c.as_ref()).collect()
    }
}

pub mod anthropic_experimental;
pub use anthropic_experimental::AnthropicExperimentalConnector;
