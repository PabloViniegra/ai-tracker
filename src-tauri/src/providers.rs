use crate::domain::{
    AnthropicConnectionState, Confidence, OpenAiConnectionState, ProviderCapabilities, ProviderId,
    ProviderStatus, ProviderSummary, UsageSource,
};
use crate::storage::{ProviderSourceState, StoredUsageSnapshot};

fn official(tokens: bool, cost: bool, quota: bool) -> ProviderCapabilities {
    ProviderCapabilities {
        tokens,
        cost,
        quota,
        realtime: false,
        historical: true,
    }
}

fn empty_provider(
    id: ProviderId,
    status: ProviderStatus,
    source: UsageSource,
    confidence: Confidence,
    capabilities: ProviderCapabilities,
) -> ProviderSummary {
    ProviderSummary {
        id,
        name: id.display_name().to_string(),
        status,
        source,
        confidence,
        capabilities,
        daily_tokens: 0,
        weekly_tokens: 0,
        cost_usd: None,
        quota_used: None,
        quota_limit: None,
        last_sync: None,
    }
}

pub fn base_provider_catalog() -> Vec<ProviderSummary> {
    vec![
        empty_provider(
            ProviderId::Openai,
            ProviderStatus::NeedsCredentials,
            UsageSource::OfficialApi,
            Confidence::High,
            official(true, true, true),
        ),
        empty_provider(
            ProviderId::Anthropic,
            ProviderStatus::NeedsCredentials,
            UsageSource::OfficialApi,
            Confidence::High,
            official(true, true, false),
        ),
    ]
}

pub fn merge_openai_summary(
    providers: &mut [ProviderSummary],
    connection: &OpenAiConnectionState,
    usage: &[StoredUsageSnapshot],
) {
    let Some(openai) = providers
        .iter_mut()
        .find(|provider| provider.id == ProviderId::Openai)
    else {
        return;
    };

    if !connection.has_credentials {
        openai.status = ProviderStatus::NeedsCredentials;
        return;
    }

    openai.status = ProviderStatus::Connected;
    openai.last_sync = connection.last_sync_at.clone();

    if let Some(latest) = usage.last() {
        openai.daily_tokens = latest.total_tokens;
    }

    openai.weekly_tokens = usage.iter().map(|snapshot| snapshot.total_tokens).sum();

    let weekly_cost = usage
        .iter()
        .fold(0.0, |sum, snapshot| sum + snapshot.cost_usd.unwrap_or(0.0));
    if weekly_cost > 0.0 {
        openai.cost_usd = Some(weekly_cost);
    }

    openai.quota_used = usage
        .last()
        .map(|snapshot| snapshot.quota_used)
        .filter(|value| *value > 0);
    openai.quota_limit = usage.last().and_then(|snapshot| snapshot.quota_limit);
}

pub fn merge_anthropic_summary(
    providers: &mut [ProviderSummary],
    connection: &AnthropicConnectionState,
    usage: &[StoredUsageSnapshot],
) {
    let Some(anthropic) = providers
        .iter_mut()
        .find(|provider| provider.id == ProviderId::Anthropic)
    else {
        return;
    };

    if !connection.has_credentials {
        anthropic.status = ProviderStatus::NeedsCredentials;
        return;
    }

    anthropic.status = ProviderStatus::Connected;
    anthropic.last_sync = connection.last_sync_at.clone();

    if let Some(latest) = usage.last() {
        anthropic.daily_tokens = latest.total_tokens;
    }

    anthropic.weekly_tokens = usage.iter().map(|snapshot| snapshot.total_tokens).sum();

    let weekly_cost = usage
        .iter()
        .fold(0.0, |sum, snapshot| sum + snapshot.cost_usd.unwrap_or(0.0));
    if weekly_cost > 0.0 {
        anthropic.cost_usd = Some(weekly_cost);
    }

    anthropic.quota_used = usage
        .last()
        .map(|snapshot| snapshot.quota_used)
        .filter(|value| *value > 0);
    anthropic.quota_limit = usage.last().and_then(|snapshot| snapshot.quota_limit);
}

pub fn merge_anthropic_source_info(
    providers: &mut [ProviderSummary],
    source_state: &ProviderSourceState,
) {
    let Some(anthropic) = providers
        .iter_mut()
        .find(|provider| provider.id == ProviderId::Anthropic)
    else {
        return;
    };

    match source_state.confidence.as_str() {
        "high" => anthropic.confidence = Confidence::High,
        "medium" => anthropic.confidence = Confidence::Medium,
        "low" => anthropic.confidence = Confidence::Low,
        _ => {}
    }

    if anthropic.status == ProviderStatus::Connected {
        return;
    }

    anthropic.source = match source_state.source_kind.as_str() {
        "official_api" => UsageSource::OfficialApi,
        "experimental_local_oauth" => UsageSource::ExperimentalLocalOauth,
        "manual" => UsageSource::Manual,
        _ => anthropic.source,
    };

    if source_state.usage_access {
        anthropic.status = ProviderStatus::Connected;
        if anthropic.last_sync.is_none() {
            anthropic.last_sync = source_state.last_success_at.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anthropic_provider(status: ProviderStatus, source: UsageSource) -> ProviderSummary {
        ProviderSummary {
            id: ProviderId::Anthropic,
            name: "Anthropic".to_string(),
            status,
            source,
            confidence: Confidence::High,
            capabilities: ProviderCapabilities {
                tokens: true,
                cost: true,
                quota: false,
                realtime: false,
                historical: true,
            },
            daily_tokens: 0,
            weekly_tokens: 0,
            cost_usd: None,
            quota_used: None,
            quota_limit: None,
            last_sync: None,
        }
    }

    #[test]
    fn keeps_official_source_when_already_connected() {
        let mut providers = vec![anthropic_provider(
            ProviderStatus::Connected,
            UsageSource::OfficialApi,
        )];
        let state = ProviderSourceState {
            source_kind: "experimental_local_oauth".to_string(),
            confidence: "medium".to_string(),
            usage_access: true,
            last_success_at: Some("2026-01-01T00:00:00Z".to_string()),
        };

        merge_anthropic_source_info(&mut providers, &state);
        let provider = providers.first().expect("provider missing");

        assert!(matches!(provider.source, UsageSource::OfficialApi));
        assert!(matches!(provider.status, ProviderStatus::Connected));
        assert!(matches!(provider.confidence, Confidence::Medium));
    }

    #[test]
    fn uses_experimental_source_when_official_not_connected() {
        let mut providers = vec![anthropic_provider(
            ProviderStatus::NeedsCredentials,
            UsageSource::OfficialApi,
        )];
        let state = ProviderSourceState {
            source_kind: "experimental_local_oauth".to_string(),
            confidence: "low".to_string(),
            usage_access: true,
            last_success_at: Some("2026-01-02T00:00:00Z".to_string()),
        };

        merge_anthropic_source_info(&mut providers, &state);
        let provider = providers.first().expect("provider missing");

        assert!(matches!(provider.source, UsageSource::ExperimentalLocalOauth));
        assert!(matches!(provider.status, ProviderStatus::Connected));
        assert!(matches!(provider.confidence, Confidence::Low));
        assert_eq!(provider.last_sync.as_deref(), Some("2026-01-02T00:00:00Z"));
    }
}
