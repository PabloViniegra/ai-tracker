# Subscription Domain & Connector Architecture Design

## Problem

The current `ai-tracker` architecture uses a singleton settings record per provider and day-level usage snapshots keyed only by `(provider_id, usage_date)`. This prevents:
- Multiple accounts, workspaces, orgs, or projects per provider.
- Distinguishing official API data from experimental/local sources.
- Tracking subscription state, plan history, or entitlements as first-class concepts.
- Reconciling conflicting signals from different sources for the same scope.

Anthropic currently validates credentials but cannot ingest usage via official APIs. OpenAI requires admin-level org access for real usage/cost data. The product needs a formal subscription domain that supports both official and experimental sources without mixing them or corrupting state.

## Approved Direction

Build a complete local subscription domain with progressive connector activation. The architecture introduces:
- A connector abstraction isolating provider-specific knowledge.
- A normalization layer converting all sources to canonical events/snapshots.
- A formal subscription domain with accounts, scopes, subscriptions, entitlements, and usage.
- Reconciliation rules selecting the best visible truth per scope by source precedence.
- An Anthropic experimental connector based on local OAuth patterns, explicitly labeled and non-destructive.

## Architecture

### 4-Layer Design

```
Vue Dashboard
  -> Tauri commands
    -> Presentation layer (aggregated views)
      -> Subscription domain (entities, reconciliation)
        -> Normalization layer (canonical shapes)
          -> Connector layer (provider-specific adapters)
            -> Official APIs
            -> Experimental local OAuth
            -> Experimental local CLI
            -> Manual input
```

**Layer responsibilities:**

1. **Connector layer**: Each integration lives behind a common Rust trait. A connector exposes one or more source kinds per provider. Provider-specific knowledge stays encapsulated.

2. **Normalization layer**: All connectors convert responses to canonical events and snapshots: accounts detected, scopes detected, subscription state, limits/quota, usage metrics, plan signals, provenance, and confidence.

3. **Subscription domain**: Persistent entities representing the real subscription landscape: `provider_account`, `workspace_scope`, `subscription_state`, `entitlement`, `usage_snapshot`, `sync_run`, `connection_source`, and optional `raw_observation`.

4. **Presentation layer**: The dashboard reads aggregated views over the normalized domain, not raw provider settings. The frontend does not depend on how each connector obtains data.

**Core rule:**
- Connectors discover facts.
- Normalization converts them to comparable records.
- The domain decides which state is currently visible.
- The UI only consumes summaries.

## Data Model

### 1. provider_account

Represents a detected identity within a provider.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `provider_id` | TEXT NOT NULL | `openai`, `anthropic` |
| `external_account_id` | TEXT | Provider-side identifier (nullable for anonymous/local) |
| `display_name` | TEXT | User-facing label |
| `email` | TEXT | If available from source |
| `status` | TEXT NOT NULL | `active`, `inactive`, `unknown` |
| `first_seen_at` | TEXT NOT NULL | ISO 8601 |
| `last_seen_at` | TEXT NOT NULL | ISO 8601 |

Unique constraint: `(provider_id, external_account_id)` where `external_account_id IS NOT NULL`.

### 2. workspace_scope

Represents the actual boundary where usage, limits, or plans apply.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `provider_account_id` | INTEGER NOT NULL | FK -> provider_account |
| `scope_type` | TEXT NOT NULL | `personal`, `organization`, `project`, `workspace` |
| `external_scope_id` | TEXT | Provider-side scope identifier |
| `parent_scope_id` | INTEGER | FK -> workspace_scope (self-referential) |
| `display_name` | TEXT | User-facing label |
| `is_default` | INTEGER NOT NULL DEFAULT 0 | Boolean flag |
| `first_seen_at` | TEXT NOT NULL | ISO 8601 |
| `last_seen_at` | TEXT NOT NULL | ISO 8601 |

Unique constraint: `(provider_account_id, scope_type, external_scope_id)` where `external_scope_id IS NOT NULL`.

### 3. connection_source

Describes where a signal came from and whether the source is functional.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `provider_id` | TEXT NOT NULL | `openai`, `anthropic` |
| `source_kind` | TEXT NOT NULL | `official_api`, `experimental_local_oauth`, `experimental_local_cli`, `manual` |
| `credential_ref` | TEXT | Keyring service name or logical path (never the secret itself) |
| `source_label` | TEXT | User-facing description |
| `is_enabled` | INTEGER NOT NULL DEFAULT 1 | Boolean flag |
| `last_validated_at` | TEXT | ISO 8601 |
| `last_error` | TEXT | Last error message |
| `last_success_at` | TEXT | ISO 8601 |

### 4. subscription_state

Current or historical subscription state detected for a workspace scope.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `workspace_scope_id` | INTEGER NOT NULL | FK -> workspace_scope |
| `connection_source_id` | INTEGER NOT NULL | FK -> connection_source |
| `plan_code` | TEXT | Machine-readable plan identifier |
| `plan_label` | TEXT | Human-readable plan name |
| `billing_status` | TEXT | `active`, `trial`, `expired`, `cancelled`, `unknown` |
| `usage_access` | INTEGER NOT NULL DEFAULT 0 | Boolean: source can retrieve usage data |
| `detected_at` | TEXT NOT NULL | ISO 8601 |
| `effective_from` | TEXT | ISO 8601 |
| `effective_to` | TEXT | ISO 8601 |
| `is_current` | INTEGER NOT NULL DEFAULT 1 | Boolean: this is the preferred visible state |
| `confidence` | TEXT NOT NULL | `high`, `medium`, `low` |

Only one `subscription_state` per `workspace_scope` should have `is_current = 1` for a given `connection_source`. Reconciliation rules determine which one wins when multiple sources report different states.

### 5. entitlement

Capabilities or limits detected and associated with a subscription.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `subscription_state_id` | INTEGER NOT NULL | FK -> subscription_state |
| `entitlement_kind` | TEXT NOT NULL | `token_window`, `weekly_quota`, `credits_balance`, `code_reviews`, `model_limit` |
| `metric_key` | TEXT | Sub-identifier within kind |
| `used_value` | REAL | Current usage |
| `limit_value` | REAL | Cap or limit |
| `unit` | TEXT | `percent`, `tokens`, `dollars`, `count` |
| `window_seconds` | INTEGER | Duration of the enforcement window |
| `resets_at` | TEXT | ISO 8601 |
| `raw_label` | TEXT | Original provider label |

### 6. usage_snapshot

Normalized time-series usage data.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `workspace_scope_id` | INTEGER NOT NULL | FK -> workspace_scope |
| `connection_source_id` | INTEGER NOT NULL | FK -> connection_source |
| `usage_date` | TEXT NOT NULL | YYYY-MM-DD |
| `model_key` | TEXT | Model identifier (nullable for aggregated) |
| `input_tokens` | INTEGER NOT NULL DEFAULT 0 | |
| `output_tokens` | INTEGER NOT NULL DEFAULT 0 | |
| `cached_tokens` | INTEGER NOT NULL DEFAULT 0 | |
| `total_tokens` | INTEGER NOT NULL DEFAULT 0 | |
| `request_count` | INTEGER NOT NULL DEFAULT 0 | |
| `cost_usd` | REAL | |
| `quota_used` | INTEGER NOT NULL DEFAULT 0 | |
| `quota_limit` | INTEGER | |
| `confidence` | TEXT NOT NULL | `high`, `medium`, `low` |

Recommended unique constraint: `(workspace_scope_id, connection_source_id, usage_date, model_key)`.

This replaces the old `(provider_id, usage_date)` key which was too coarse.

### 7. sync_run

Audit trail for connector execution.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `connection_source_id` | INTEGER NOT NULL | FK -> connection_source |
| `provider_id` | TEXT NOT NULL | |
| `status` | TEXT NOT NULL | `success`, `warning`, `error` |
| `started_at` | TEXT NOT NULL | ISO 8601 |
| `finished_at` | TEXT | ISO 8601 |
| `message` | TEXT | Summary or error |
| `discovered_accounts` | INTEGER DEFAULT 0 | |
| `discovered_scopes` | INTEGER DEFAULT 0 | |
| `wrote_snapshots` | INTEGER DEFAULT 0 | |

### 8. raw_observation (optional but recommended)

Stores raw payload fragments for debugging experimental connectors.

| Field | Type | Notes |
|---|---|---|
| `id` | INTEGER PRIMARY KEY | Auto-increment |
| `connection_source_id` | INTEGER NOT NULL | FK -> connection_source |
| `observation_kind` | TEXT NOT NULL | `usage_response`, `subscription_response`, `auth_response` |
| `captured_at` | TEXT NOT NULL | ISO 8601 |
| `payload_json` | TEXT NOT NULL | Redacted JSON |
| `redacted` | INTEGER NOT NULL DEFAULT 1 | Boolean: must be true for experimental sources |

**Constraint:** Never store raw tokens, API keys, or refresh tokens. Redaction is mandatory.

### Relationships

```
provider_account (1) -> (N) workspace_scope
workspace_scope (1) -> (N) subscription_state
connection_source (1) -> (N) subscription_state
subscription_state (1) -> (N) entitlement
workspace_scope (1) -> (N) usage_snapshot
connection_source (1) -> (N) usage_snapshot
connection_source (1) -> (N) sync_run
connection_source (1) -> (N) raw_observation
```

## Sync Flow

Each `connection_source` syncs independently:

1. **Trigger**: Scheduler or manual refresh initiates a `sync_run`.
2. **Validation**: Connector validates credentials and determines whether the source has real access to usage, plan, or both.
3. **Discovery**: Connector returns a normalized package containing:
   - Detected accounts
   - Detected scopes
   - Subscription states
   - Entitlements
   - Usage snapshots
   - Optional raw observations
4. **Upsert**: Normalization layer writes by stable identity:
   - `provider_account` by `provider_id + external_account_id`
   - `workspace_scope` by `provider_account_id + scope_type + external_scope_id`
   - `connection_source` by `provider + source_kind + credential_ref`
   - `usage_snapshot` by `workspace_scope_id + connection_source_id + usage_date + model_key`
5. **Reconciliation**: Domain layer determines which `subscription_state` is `is_current` per `workspace_scope`.

## Reconciliation Rules

Source precedence (highest to lowest):
1. `official_api`
2. `experimental_local_oauth`
3. `experimental_local_cli`
4. `manual`

Rules:
- Truth is scoped per `workspace_scope`, not per provider globally.
- When two sources report different `plan_label` for the same scope, both persist but only one is `is_current = true` (highest precedence wins).
- `usage_access` and `billing_status` are computed per source, not per provider.
- The UI can show: principal source, secondary source available, experimental data flag.
- A failed sync does not overwrite the last good state.
- Experimental connectors fail as `warning`, never as state corruption.

This solves the current problem where `connected = true` even when no useful usage data exists.

## Anthropic Experimental Connector

Based on `openusage` patterns, not its non-existent billing domain.

### Implementation phases

1. Add connector `anthropic_experimental_local_oauth`.
2. Read local Claude Code credentials:
   - `~/.claude/.credentials.json`
   - macOS keychain fallback (when platform supports it)
3. Proactively refresh OAuth token if expired or near expiry.
4. Query the reverse-engineered usage endpoint: `GET https://api.anthropic.com/api/oauth/usage` with `anthropic-beta: oauth-2025-04-20` header.
5. Map results:
   - Detected account -> `provider_account`
   - Personal scope (default) -> `workspace_scope`
   - `subscriptionType` / `rateLimitTier` -> `subscription_state`
   - `five_hour`, `seven_day` windows -> `entitlement`
   - `extra_usage` -> `entitlement`
   - Additional local signals -> `raw_observation` or `usage_snapshot`

### Explicit constraints

- Must be labeled `experimental` in all UI and internal metadata.
- Must NOT overwrite an `official_api` source for the same scope.
- Must NOT expose secrets in logs, `raw_observation`, or error messages.
- Must degrade gracefully with clear messages if Anthropic changes the endpoint.
- Token refresh follows the same pattern as `openusage`: proactive 5-minute buffer, reactive on 401.

### Mapped fields from Anthropic OAuth usage response

| Response field | Target |
|---|---|
| `five_hour.utilization` | entitlement (token_window, 5h) |
| `five_hour.resets_at` | entitlement.resets_at |
| `seven_day.utilization` | entitlement (weekly_quota, 7d) |
| `seven_day.resets_at` | entitlement.resets_at |
| `seven_day_sonnet.utilization` | entitlement (model_limit, sonnet) |
| `seven_day_omelette.utilization` | entitlement (model_limit, claude_design) |
| `extra_usage.used_credits` | entitlement (credits_balance, spent) |
| `extra_usage.monthly_limit` | entitlement (credits_balance, cap) |
| `subscriptionType` | subscription_state.plan_code |
| `rateLimitTier` | subscription_state (derived plan_label suffix) |

## Error Handling & Security

- **Never persist raw OAuth tokens, API keys, or refresh tokens in SQLite.**
- `credential_ref` points to keyring service name or logical path only.
- `raw_observation.payload_json` must have all secrets redacted before storage.
- If a sync fails, the previous good state remains visible.
- Experimental connectors produce `warning` status on failure, not `error` that corrupts state.
- Rate limiting: connectors must respect `Retry-After` headers and implement exponential backoff.
- Timeout: all HTTP calls must have explicit timeouts (10-15 seconds).

## Testing Strategy

- Unit tests for reconciliation logic and source precedence.
- SQLite migration tests from current schema to new schema.
- Normalized fixtures per provider (official and experimental).
- Secret redaction tests for `raw_observation`.
- Tests verifying experimental sources never overwrite official sources.
- Connector mock tests with fixture responses.
- Edge cases: token expiry, 429 rate limits, empty responses, malformed JSON.

## Migration Path

### Phase 0: Schema migration
- Create new tables alongside existing ones.
- Migrate existing `openai_settings` and `anthropic_settings` into `provider_account`, `workspace_scope`, and `connection_source` records.
- Migrate existing `usage_snapshots` into new `usage_snapshot` table with proper scope/source linkage.
- Keep old tables readable during transition; remove after verification.

### Phase 1: Connector abstraction
- Define `ProviderConnector` trait in Rust.
- Wrap existing `openai.rs` as `OpenAiOfficialConnector`.
- Wrap existing `anthropic.rs` as `AnthropicOfficialConnector` (validation only, no usage).
- Dashboard reads from new domain views, falling back to old tables if migration incomplete.

### Phase 2: Anthropic experimental
- Implement `AnthropicExperimentalLocalOAuthConnector`.
- Add credential discovery, refresh, and usage fetch.
- Map to domain entities.
- Gate behind feature flag or explicit user opt-in.

### Phase 3: Reconciliation & UI
- Implement reconciliation engine.
- Update dashboard to show multi-source, multi-scope views.
- Add source labels and confidence indicators to all metrics.
- Remove legacy settings tables.

## Component Boundaries

### Rust backend

| Module | Responsibility |
|---|---|
| `domain/` | Serializable models for all entities |
| `connectors/` | Trait definition + per-provider implementations |
| `normalization/` | Canonical shape conversion |
| `reconciliation/` | Source precedence and state selection |
| `storage/` | SQLite schema, migrations, CRUD operations |
| `security/` | Keyring operations, secret redaction |
| `lib.rs` | Tauri commands, orchestration |

### Vue frontend

| Component | Responsibility |
|---|---|
| `App.vue` | Shell composition, data loading |
| `components/dashboard/ProviderGrid.vue` | Multi-source provider cards |
| `components/dashboard/SubscriptionDetail.vue` | Scope-level subscription view |
| `components/dashboard/SourceIndicator.vue` | Source/confidence badges |
| `composables/useDashboardData.ts` | Tauri command integration |
| `types/usage.ts` | TypeScript contracts matching Rust payloads |

## Out of Scope (for this change)

- Cloud backend or multi-device sync.
- Stripe/billing integration.
- Automated payment or invoice tracking.
- Real-time streaming usage (polling remains the model).
- Provider auto-discovery beyond configured sources.
- Cross-provider aggregation or budget alerts.
