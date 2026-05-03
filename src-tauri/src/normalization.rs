use crate::connectors::SyncPackage;
use crate::domain::{ProviderAccountRecord, WorkspaceScopeRecord, SubscriptionStateRecord, EntitlementRecord, UsageSnapshotV2Record, RawObservationRecord};
use crate::storage;
use std::path::Path;

pub fn normalize_and_store(path: &Path, source_id: i64, pkg: SyncPackage) -> Result<(), String> {
    for account in &pkg.accounts {
        let record = ProviderAccountRecord {
            id: None,
            provider_id: account.provider_id.clone(),
            external_account_id: account.external_account_id.clone(),
            display_name: account.display_name.clone(),
            email: account.email.clone(),
            status: account.status.clone(),
            first_seen_at: account.first_seen_at.clone(),
            last_seen_at: account.last_seen_at.clone(),
        };
        let account_id = storage::upsert_provider_account(path, &record)?;
        let scope = WorkspaceScopeRecord {
            id: None,
            provider_account_id: account_id,
            scope_type: "personal".to_string(),
            external_scope_id: None,
            parent_scope_id: None,
            display_name: "Personal".to_string(),
            is_default: true,
            first_seen_at: account.first_seen_at.clone(),
            last_seen_at: account.last_seen_at.clone(),
        };
        let scope_id = storage::upsert_workspace_scope(path, &scope)?;

        for subscription in &pkg.subscriptions {
            let sub_record = SubscriptionStateRecord {
                id: None,
                workspace_scope_id: scope_id,
                connection_source_id: source_id,
                plan_code: subscription.plan_code.clone(),
                plan_label: subscription.plan_label.clone(),
                billing_status: subscription.billing_status.clone(),
                usage_access: subscription.usage_access,
                detected_at: subscription.detected_at.clone(),
                effective_from: subscription.effective_from.clone(),
                effective_to: subscription.effective_to.clone(),
                is_current: subscription.is_current,
                confidence: subscription.confidence.clone(),
            };
            let sub_id = storage::upsert_subscription_state(path, &sub_record)?;

            let ents: Vec<EntitlementRecord> = pkg
                .entitlements
                .iter()
                .filter(|e| e.subscription_state_id == 0)
                .map(|e| EntitlementRecord {
                    id: None,
                    subscription_state_id: sub_id,
                    entitlement_kind: e.entitlement_kind.clone(),
                    metric_key: e.metric_key.clone(),
                    used_value: e.used_value,
                    limit_value: e.limit_value,
                    unit: e.unit.clone(),
                    window_seconds: e.window_seconds,
                    resets_at: e.resets_at.clone(),
                    raw_label: e.raw_label.clone(),
                })
                .collect();
            if !ents.is_empty() {
                storage::upsert_entitlements(path, &ents)?;
            }
        }
    }

    for snapshot in &pkg.usage_snapshots {
        let rec = UsageSnapshotV2Record {
            id: None,
            workspace_scope_id: snapshot.workspace_scope_id,
            connection_source_id: source_id,
            usage_date: snapshot.usage_date.clone(),
            model_key: snapshot.model_key.clone(),
            input_tokens: snapshot.input_tokens,
            output_tokens: snapshot.output_tokens,
            cached_tokens: snapshot.cached_tokens,
            total_tokens: snapshot.total_tokens,
            request_count: snapshot.request_count,
            cost_usd: snapshot.cost_usd,
            quota_used: snapshot.quota_used,
            quota_limit: snapshot.quota_limit,
            confidence: snapshot.confidence.clone(),
        };
        storage::insert_usage_snapshots_v2(path, &[rec])?;
    }

    for obs in &pkg.raw_observations {
        let rec = RawObservationRecord {
            id: None,
            connection_source_id: source_id,
            observation_kind: obs.observation_kind.clone(),
            captured_at: obs.captured_at.clone(),
            payload_json: obs.payload_json.clone(),
            redacted: obs.redacted,
        };
        storage::insert_raw_observation(path, &rec)?;
    }

    Ok(())
}