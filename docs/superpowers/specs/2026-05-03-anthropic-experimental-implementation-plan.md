# Implementation Plan: Anthropic Experimental Connector

## Overview

This plan implements the Anthropic experimental connector based on the `openusage` reverse-engineered patterns, integrated into the new subscription domain architecture. It follows a phased approach that minimizes risk while delivering incremental value.

**Reference documents:**
- Design spec: `docs/superpowers/specs/2026-05-03-subscription-domain-connector-design.md`
- OpenUsage Claude plugin: `plugins/claude/plugin.js` (external reference)

## Phase 0: Infrastructure & Schema

### Task 0.1: Define connector trait
**Files:** `src-tauri/src/connectors/mod.rs` (new)

Define the common connector interface:
```rust
pub trait ProviderConnector {
    fn provider_id(&self) -> &'static str;
    fn source_kind(&self) -> ConnectionSourceKind;
    async fn validate(&self, credential_ref: &str) -> Result<ValidationResult, ConnectorError>;
    async fn sync(&self, credential_ref: &str) -> Result<SyncPackage, ConnectorError>;
}

pub struct ValidationResult {
    pub valid: bool,
    pub can_access_usage: bool,
    pub can_access_subscription: bool,
    pub account_hint: Option<AccountHint>,
}

pub struct SyncPackage {
    pub accounts: Vec<DetectedAccount>,
    pub scopes: Vec<DetectedScope>,
    pub subscriptions: Vec<DetectedSubscription>,
    pub entitlements: Vec<DetectedEntitlement>,
    pub usage_snapshots: Vec<DetectedUsageSnapshot>,
    pub raw_observations: Vec<DetectedRawObservation>,
}
```

### Task 0.2: Extend storage schema
**Files:** `src-tauri/src/storage.rs`

Add migration that creates new tables alongside existing ones (no destructive changes yet):
- `provider_account`
- `workspace_scope`
- `connection_source`
- `subscription_state`
- `entitlement`
- `usage_snapshot_v2` (temporary name during migration)
- `sync_run`
- `raw_observation`

Keep old tables intact. Provide migration functions that copy existing data into new schema with default values for new required fields.

### Task 0.3: Implement domain models
**Files:** `src-tauri/src/domain.rs` (extend)

Add Rust structs mirroring the new entities. Keep existing models for backward compatibility during migration.

### Task 0.4: Create security utilities
**Files:** `src-tauri/src/security.rs` (extend)

Add functions for:
- Reading Anthropic credentials from `~/.claude/.credentials.json`
- Reading from keychain (platform-gated, macOS only initially)
- Writing credentials back after OAuth refresh
- Redacting secrets from JSON payloads for `raw_observation`

## Phase 1: Anthropic Experimental Connector

### Task 1.1: Implement credential discovery
**Files:** `src-tauri/src/connectors/anthropic_experimental.rs` (new)

Implement credential loading following `openusage` patterns:
1. Check `CLAUDE_CONFIG_DIR` env var, default to `~/.claude`
2. Read `.credentials.json` from that directory
3. Parse JSON looking for `claudeAiOauth` object with:
   - `accessToken`
   - `refreshToken`
   - `expiresAt` (unix ms)
   - `scopes`
   - `subscriptionType`
   - `rateLimitTier`
4. Fallback to keychain if file not found (macOS only)
5. Handle hex-encoded keychain payloads (de-hex then UTF-8 decode)

### Task 1.2: Implement OAuth refresh
**Files:** `src-tauri/src/connectors/anthropic_experimental.rs`

Implement token refresh following `openusage` patterns:
- Refresh URL: `https://platform.claude.com/v1/oauth/token`
- Client ID: `9d1c250a-e61b-44d9-88ed-5944d1962f5e`
- Refresh 5 minutes before expiry (`REFRESH_BUFFER_MS = 5 * 60 * 1000`)
- POST with JSON body: `grant_type=refresh_token`, `refresh_token`, `client_id`, `scope`
- Update `accessToken`, `refreshToken`, `expiresAt` in credentials
- Persist updated credentials back to same source (file or keychain)
- Handle error codes: `invalid_grant` -> session expired

### Task 1.3: Implement usage fetch
**Files:** `src-tauri/src/connectors/anthropic_experimental.rs`

Implement usage API call:
- URL: `https://api.anthropic.com/api/oauth/usage`
- Headers:
  - `Authorization: Bearer <access_token>`
  - `Accept: application/json`
  - `Content-Type: application/json`
  - `anthropic-beta: oauth-2025-04-20`
  - `User-Agent: claude-code/2.1.69`
- Implement retry-on-auth: if 401, refresh token and retry once
- Parse response into `AnthropicUsageResponse` struct

### Task 1.4: Implement normalization
**Files:** `src-tauri/src/connectors/anthropic_experimental.rs`

Map Anthropic response to canonical `SyncPackage`:

```rust
// Account detection
DetectedAccount {
    external_account_id: None, // OAuth doesn't expose stable account ID
    display_name: "Claude Code Account".to_string(),
    email: None,
    status: "active",
}

// Scope detection
DetectedScope {
    scope_type: "personal",
    external_scope_id: None,
    display_name: "Personal",
    is_default: true,
}

// Subscription state
DetectedSubscription {
    plan_code: creds.subscription_type.clone(),
    plan_label: format_plan_label(&creds.subscription_type, &creds.rate_limit_tier),
    billing_status: "active",
    usage_access: true,
    confidence: "medium", // experimental source
}

// Entitlements from usage response
// five_hour -> token_window, 5h window
// seven_day -> weekly_quota, 7d window
// seven_day_sonnet -> model_limit, sonnet
// seven_day_omelette -> model_limit, claude_design
// extra_usage -> credits_balance
```

### Task 1.5: Register connector
**Files:** `src-tauri/src/connectors/mod.rs`, `src-tauri/src/lib.rs`

Add the experimental connector to the connector registry:
- Only enabled when Anthropic credentials exist locally
- Marked with `source_kind: experimental_local_oauth`
- Lower precedence than official API (when available in future)

## Phase 2: Integration & Storage

### Task 2.1: Implement upsert logic
**Files:** `src-tauri/src/storage.rs` (extend)

Add CRUD operations for new entities:
- `upsert_provider_account`
- `upsert_workspace_scope`
- `upsert_connection_source`
- `upsert_subscription_state`
- `upsert_entitlements`
- `insert_usage_snapshots_v2`
- `insert_sync_run`
- `insert_raw_observation`

Implement stable identity matching:
- Account by `provider_id + external_account_id` (or fingerprint for local accounts)
- Scope by `provider_account_id + scope_type + external_scope_id`
- Source by `provider_id + source_kind + credential_ref`

### Task 2.2: Implement reconciliation
**Files:** `src-tauri/src/reconciliation.rs` (new)

Implement the reconciliation engine:
```rust
pub fn reconcile_subscriptions(
    conn: &Connection,
    workspace_scope_id: i64,
) -> Result<(), String> {
    // Get all subscription_states for this scope
    // Sort by source precedence: official_api > experimental_local_oauth > experimental_local_cli > manual
    // Mark highest as is_current = 1, others as is_current = 0
    // Handle ties by detected_at (most recent wins)
}
```

### Task 2.3: Wire up Tauri commands
**Files:** `src-tauri/src/lib.rs`

Add or modify commands:
- `get_dashboard_snapshot`: Read from new schema, fallback to old
- `sync_anthropic_experimental`: Trigger experimental connector sync
- `get_anthropic_sources`: List available sources (official + experimental)
- `enable_anthropic_experimental`: Opt-in flag for experimental source

## Phase 3: Frontend Updates

### Task 3.1: Update TypeScript types
**Files:** `src/types/usage.ts`

Add types for:
- `ProviderAccount`
- `WorkspaceScope`
- `ConnectionSource`
- `SubscriptionState`
- `Entitlement`
- `UsageSnapshotV2`
- `SyncRun`

### Task 3.2: Update dashboard composable
**Files:** `src/composables/useDashboardData.ts`

- Fetch from new Tauri commands
- Aggregate multiple sources per provider
- Show source badges and confidence indicators
- Handle experimental source opt-in

### Task 3.3: Update Anthropic setup panel
**Files:** `src/components/dashboard/AnthropicSetupPanel.vue`

- Show official API key input (current behavior)
- Add experimental source opt-in section
- Explain what experimental source accesses (local Claude Code credentials)
- Show detected local credentials status
- Allow manual refresh of experimental source

### Task 3.4: Add source indicators
**Files:** `src/components/dashboard/SourceIndicator.vue` (new)

Small component showing:
- Source badge: `official`, `experimental`, `manual`
- Confidence indicator: `high`, `medium`, `low`
- Last sync time
- Warning icon if source is experimental

## Phase 4: Testing & Validation

### Task 4.1: Unit tests for connector
**Files:** `src-tauri/src/connectors/anthropic_experimental.rs` (tests inline)

Test:
- Credential discovery (file exists, file missing, keychain fallback)
- JSON parsing (normal, hex-encoded, malformed)
- OAuth refresh (success, expired token, invalid grant)
- Usage fetch (success, 401 triggers refresh, 429 rate limit)
- Response normalization (all fields present, partial, empty)

### Task 4.2: Integration tests for reconciliation
**Files:** `src-tauri/src/reconciliation.rs` (tests inline)

Test:
- Official API wins over experimental
- Experimental wins over manual
- Same precedence: most recent wins
- Failed sync preserves previous state
- Multiple scopes per account

### Task 4.3: Secret redaction tests
**Files:** `src-tauri/src/security.rs` (tests inline)

Test:
- Bearer tokens removed from JSON
- Refresh tokens removed
- API keys removed
- Other data preserved
- Hex-encoded secrets also handled

### Task 4.4: Migration tests
**Files:** Test in CI/build step

- Create old schema, insert sample data
- Run migration
- Verify data in new schema
- Verify old schema still readable

### Task 4.5: End-to-end validation
- Manual test with real Claude Code credentials
- Verify usage data appears in dashboard
- Verify source badge shows "experimental"
- Verify experimental source does not overwrite official (if both present)
- Test credential refresh flow
- Test graceful degradation when Anthropic changes API

## Rollout Strategy

### Feature gating
- Experimental connector is disabled by default
- User must explicitly enable in Anthropic setup panel
- UI shows warning banner when experimental source is active
- Easy rollback: disable source, delete connection_source record

### Gradual migration
- Phase 0-1: Backend-only, no UI changes
- Phase 2: Add behind feature flag
- Phase 3: Enable for testing
- Phase 4: General availability after validation

### Monitoring
- Log all sync_run outcomes
- Track raw_observation for debugging (with redaction)
- Monitor for rate limiting or API changes
- Alert if experimental source fails consistently

## File Summary

### New files
```
src-tauri/src/connectors/mod.rs
src-tauri/src/connectors/anthropic_experimental.rs
src-tauri/src/normalization.rs
src-tauri/src/reconciliation.rs
src/components/dashboard/SourceIndicator.vue
```

### Modified files
```
src-tauri/src/domain.rs
src-tauri/src/storage.rs
src-tauri/src/security.rs
src-tauri/src/lib.rs
src-tauri/src/providers.rs
src/types/usage.ts
src/composables/useDashboardData.ts
src/components/dashboard/AnthropicSetupPanel.vue
src/components/dashboard/ProviderGrid.vue
```

### Unchanged files (backward compatibility)
```
src-tauri/src/openai.rs
src/components/dashboard/OpenAiSetupPanel.vue
```

## Risk Mitigation

| Risk | Mitigation |
|---|---|
| Anthropic changes undocumented API | Graceful degradation, warning status, preserve last good state |
| Credential file format changes | Multiple fallback paths, hex decoding, robust parsing |
| OAuth token expiry during sync | Proactive refresh, reactive retry-on-auth, clear error messages |
| Rate limiting | Respect Retry-After, implement backoff, cache last successful response |
| Secret exposure | Never persist tokens in SQLite, redaction in raw_observation, keyring-only storage |
| Schema migration failure | Non-destructive migration, keep old tables, rollback capability |
| Experimental source overwrites official | Source precedence rules, reconciliation engine, explicit precedence enforcement |

## Success Criteria

- [ ] Anthropic experimental connector discovers local Claude Code credentials
- [ ] OAuth refresh works automatically when token expires
- [ ] Usage data from experimental source appears in dashboard
- [ ] Source shows as "experimental" with appropriate confidence level
- [ ] Experimental source never overwrites official API data
- [ ] Failed syncs preserve previous good state
- [ ] No secrets are persisted in SQLite or logs
- [ ] User can opt-in/out of experimental source
- [ ] All tests pass (unit, integration, migration)
- [ ] Manual validation with real Claude Code account succeeds
