use crate::domain::{
    ConnectionSourceRecord, EntitlementRecord, ProviderAccountRecord, RawObservationRecord,
    SubscriptionStateRecord, SyncRunRecord, UsageSnapshotV2Record, WorkspaceScopeRecord,
};
use rusqlite::{params, Connection};
use std::path::Path;

#[derive(Clone, Debug, Default)]
pub struct OpenAiSettingsRecord {
    pub has_credentials: bool,
    pub account_label: Option<String>,
    pub organization_id: Option<String>,
    pub project_id: Option<String>,
    pub last_validated_at: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct AnthropicSettingsRecord {
    pub has_credentials: bool,
    pub account_label: Option<String>,
    pub last_validated_at: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct StoredUsageSnapshot {
    pub usage_date: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: u64,
    pub total_tokens: u64,
    pub request_count: u64,
    pub cost_usd: Option<f64>,
    pub quota_used: u8,
    pub quota_limit: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct StoredSyncEvent {
    pub provider_id: String,
    pub provider_name: String,
    pub status: String,
    pub message: String,
    pub at: String,
}

#[derive(Clone, Debug)]
pub struct ProviderSourceState {
    pub source_kind: String,
    pub confidence: String,
    pub usage_access: bool,
    pub last_success_at: Option<String>,
}

fn open(path: &Path) -> Result<Connection, String> {
    Connection::open(path).map_err(|error| error.to_string())
}

pub fn init_database(path: &Path) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS openai_settings (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              has_credentials INTEGER NOT NULL DEFAULT 0,
              account_label TEXT,
              organization_id TEXT,
              project_id TEXT,
              last_validated_at TEXT,
              last_sync_at TEXT,
              last_error TEXT
            );

            CREATE TABLE IF NOT EXISTS anthropic_settings (
              id INTEGER PRIMARY KEY CHECK (id = 1),
              has_credentials INTEGER NOT NULL DEFAULT 0,
              account_label TEXT,
              last_validated_at TEXT,
              last_sync_at TEXT,
              last_error TEXT
            );

            CREATE TABLE IF NOT EXISTS usage_snapshots (
              provider_id TEXT NOT NULL,
              usage_date TEXT NOT NULL,
              input_tokens INTEGER NOT NULL DEFAULT 0,
              output_tokens INTEGER NOT NULL DEFAULT 0,
              cached_tokens INTEGER NOT NULL DEFAULT 0,
              total_tokens INTEGER NOT NULL DEFAULT 0,
              request_count INTEGER NOT NULL DEFAULT 0,
              cost_usd REAL,
              quota_used INTEGER NOT NULL DEFAULT 0,
              quota_limit INTEGER,
              PRIMARY KEY(provider_id, usage_date)
            );

            CREATE TABLE IF NOT EXISTS sync_events (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              provider_id TEXT NOT NULL,
              provider_name TEXT NOT NULL,
              status TEXT NOT NULL,
              message TEXT NOT NULL,
              at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS provider_accounts (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              provider_id TEXT NOT NULL,
              external_account_id TEXT,
              display_name TEXT NOT NULL,
              email TEXT,
              status TEXT NOT NULL DEFAULT 'active',
              first_seen_at TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              UNIQUE(provider_id, external_account_id)
            );

            CREATE TABLE IF NOT EXISTS workspace_scopes (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              provider_account_id INTEGER NOT NULL,
              scope_type TEXT NOT NULL,
              external_scope_id TEXT,
              parent_scope_id INTEGER,
              display_name TEXT NOT NULL,
              is_default INTEGER NOT NULL DEFAULT 0,
              first_seen_at TEXT NOT NULL,
              last_seen_at TEXT NOT NULL,
              FOREIGN KEY (provider_account_id) REFERENCES provider_accounts(id),
              UNIQUE(provider_account_id, scope_type, external_scope_id)
            );

            CREATE TABLE IF NOT EXISTS connection_sources (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              provider_id TEXT NOT NULL,
              source_kind TEXT NOT NULL,
              credential_ref TEXT,
              source_label TEXT,
              is_enabled INTEGER NOT NULL DEFAULT 1,
              last_validated_at TEXT,
              last_error TEXT,
              last_success_at TEXT,
              UNIQUE(provider_id, source_kind, credential_ref)
            );

            CREATE TABLE IF NOT EXISTS subscription_states (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              workspace_scope_id INTEGER NOT NULL,
              connection_source_id INTEGER NOT NULL,
              plan_code TEXT,
              plan_label TEXT,
              billing_status TEXT NOT NULL DEFAULT 'unknown',
              usage_access INTEGER NOT NULL DEFAULT 0,
              detected_at TEXT NOT NULL,
              effective_from TEXT,
              effective_to TEXT,
              is_current INTEGER NOT NULL DEFAULT 1,
              confidence TEXT NOT NULL DEFAULT 'high',
              FOREIGN KEY (workspace_scope_id) REFERENCES workspace_scopes(id),
              FOREIGN KEY (connection_source_id) REFERENCES connection_sources(id)
            );

            CREATE TABLE IF NOT EXISTS entitlements (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              subscription_state_id INTEGER NOT NULL,
              entitlement_kind TEXT NOT NULL,
              metric_key TEXT,
              used_value REAL,
              limit_value REAL,
              unit TEXT,
              window_seconds INTEGER,
              resets_at TEXT,
              raw_label TEXT,
              FOREIGN KEY (subscription_state_id) REFERENCES subscription_states(id)
            );

            CREATE TABLE IF NOT EXISTS usage_snapshots_v2 (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              workspace_scope_id INTEGER NOT NULL,
              connection_source_id INTEGER NOT NULL,
              usage_date TEXT NOT NULL,
              model_key TEXT,
              input_tokens INTEGER NOT NULL DEFAULT 0,
              output_tokens INTEGER NOT NULL DEFAULT 0,
              cached_tokens INTEGER NOT NULL DEFAULT 0,
              total_tokens INTEGER NOT NULL DEFAULT 0,
              request_count INTEGER NOT NULL DEFAULT 0,
              cost_usd REAL,
              quota_used INTEGER NOT NULL DEFAULT 0,
              quota_limit INTEGER,
              confidence TEXT NOT NULL DEFAULT 'high',
              FOREIGN KEY (workspace_scope_id) REFERENCES workspace_scopes(id),
              FOREIGN KEY (connection_source_id) REFERENCES connection_sources(id),
              UNIQUE(workspace_scope_id, connection_source_id, usage_date, model_key)
            );

            CREATE TABLE IF NOT EXISTS sync_runs (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              connection_source_id INTEGER NOT NULL,
              provider_id TEXT NOT NULL,
              status TEXT NOT NULL,
              started_at TEXT NOT NULL,
              finished_at TEXT,
              message TEXT,
              discovered_accounts INTEGER DEFAULT 0,
              discovered_scopes INTEGER DEFAULT 0,
              wrote_snapshots INTEGER DEFAULT 0,
              FOREIGN KEY (connection_source_id) REFERENCES connection_sources(id)
            );

            CREATE TABLE IF NOT EXISTS raw_observations (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              connection_source_id INTEGER NOT NULL,
              observation_kind TEXT NOT NULL,
              captured_at TEXT NOT NULL,
              payload_json TEXT NOT NULL,
              redacted INTEGER NOT NULL DEFAULT 1,
              FOREIGN KEY (connection_source_id) REFERENCES connection_sources(id)
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_subscription_states_scope_source_detected
              ON subscription_states(workspace_scope_id, connection_source_id, detected_at);

            CREATE UNIQUE INDEX IF NOT EXISTS idx_entitlements_subscription_kind_metric
              ON entitlements(subscription_state_id, entitlement_kind, metric_key);
            "#,
        )
        .map_err(|error| error.to_string())
}

pub fn load_openai_settings(path: &Path) -> Result<OpenAiSettingsRecord, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare(
            "SELECT has_credentials, account_label, organization_id, project_id, last_validated_at, last_sync_at, last_error FROM openai_settings WHERE id = 1",
        )
        .map_err(|error| error.to_string())?;

    let record = statement.query_row([], |row| {
        Ok(OpenAiSettingsRecord {
            has_credentials: row.get::<_, i64>(0)? > 0,
            account_label: row.get(1)?,
            organization_id: row.get(2)?,
            project_id: row.get(3)?,
            last_validated_at: row.get(4)?,
            last_sync_at: row.get(5)?,
            last_error: row.get(6)?,
        })
    });

    match record {
        Ok(settings) => Ok(settings),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(OpenAiSettingsRecord::default()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn save_openai_settings(path: &Path, settings: &OpenAiSettingsRecord) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            INSERT INTO openai_settings (id, has_credentials, account_label, organization_id, project_id, last_validated_at, last_sync_at, last_error)
            VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
              has_credentials = excluded.has_credentials,
              account_label = excluded.account_label,
              organization_id = excluded.organization_id,
              project_id = excluded.project_id,
              last_validated_at = excluded.last_validated_at,
              last_sync_at = excluded.last_sync_at,
              last_error = excluded.last_error
            "#,
            params![
                if settings.has_credentials { 1 } else { 0 },
                settings.account_label,
                settings.organization_id,
                settings.project_id,
                settings.last_validated_at,
                settings.last_sync_at,
                settings.last_error,
            ],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn load_anthropic_settings(path: &Path) -> Result<AnthropicSettingsRecord, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare(
            "SELECT has_credentials, account_label, last_validated_at, last_sync_at, last_error FROM anthropic_settings WHERE id = 1",
        )
        .map_err(|error| error.to_string())?;

    let record = statement.query_row([], |row| {
        Ok(AnthropicSettingsRecord {
            has_credentials: row.get::<_, i64>(0)? > 0,
            account_label: row.get(1)?,
            last_validated_at: row.get(2)?,
            last_sync_at: row.get(3)?,
            last_error: row.get(4)?,
        })
    });

    match record {
        Ok(settings) => Ok(settings),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AnthropicSettingsRecord::default()),
        Err(error) => Err(error.to_string()),
    }
}

pub fn save_anthropic_settings(
    path: &Path,
    settings: &AnthropicSettingsRecord,
) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            INSERT INTO anthropic_settings (id, has_credentials, account_label, last_validated_at, last_sync_at, last_error)
            VALUES (1, ?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
              has_credentials = excluded.has_credentials,
              account_label = excluded.account_label,
              last_validated_at = excluded.last_validated_at,
              last_sync_at = excluded.last_sync_at,
              last_error = excluded.last_error
            "#,
            params![
                if settings.has_credentials { 1 } else { 0 },
                settings.account_label,
                settings.last_validated_at,
                settings.last_sync_at,
                settings.last_error,
            ],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn replace_usage_snapshots(
    path: &Path,
    provider_id: &str,
    snapshots: &[StoredUsageSnapshot],
) -> Result<(), String> {
    let mut connection = open(path)?;
    let transaction = connection
        .transaction()
        .map_err(|error| error.to_string())?;

    for snapshot in snapshots {
        transaction
            .execute(
                r#"
                INSERT INTO usage_snapshots (
                  provider_id, usage_date, input_tokens, output_tokens, cached_tokens, total_tokens,
                  request_count, cost_usd, quota_used, quota_limit
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(provider_id, usage_date) DO UPDATE SET
                  input_tokens = excluded.input_tokens,
                  output_tokens = excluded.output_tokens,
                  cached_tokens = excluded.cached_tokens,
                  total_tokens = excluded.total_tokens,
                  request_count = excluded.request_count,
                  cost_usd = excluded.cost_usd,
                  quota_used = excluded.quota_used,
                  quota_limit = excluded.quota_limit
                "#,
                params![
                    provider_id,
                    snapshot.usage_date,
                    snapshot.input_tokens,
                    snapshot.output_tokens,
                    snapshot.cached_tokens,
                    snapshot.total_tokens,
                    snapshot.request_count,
                    snapshot.cost_usd,
                    snapshot.quota_used,
                    snapshot.quota_limit,
                ],
            )
            .map_err(|error| error.to_string())?;
    }

    transaction.commit().map_err(|error| error.to_string())
}

pub fn load_usage_history(
    path: &Path,
    provider_id: &str,
    limit: usize,
) -> Result<Vec<StoredUsageSnapshot>, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare(
            "SELECT usage_date, input_tokens, output_tokens, cached_tokens, total_tokens, request_count, cost_usd, quota_used, quota_limit FROM usage_snapshots WHERE provider_id = ?1 ORDER BY usage_date DESC LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;

    let mapped = statement
        .query_map(params![provider_id, limit as i64], |row| {
            Ok(StoredUsageSnapshot {
                usage_date: row.get(0)?,
                input_tokens: row.get(1)?,
                output_tokens: row.get(2)?,
                cached_tokens: row.get(3)?,
                total_tokens: row.get(4)?,
                request_count: row.get(5)?,
                cost_usd: row.get(6)?,
                quota_used: row.get(7)?,
                quota_limit: row.get(8)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut snapshots = Vec::new();
    for snapshot in mapped {
        snapshots.push(snapshot.map_err(|error| error.to_string())?);
    }
    snapshots.reverse();
    Ok(snapshots)
}

pub fn append_sync_event(path: &Path, event: &StoredSyncEvent) -> Result<(), String> {
    let connection = open(path)?;
    connection
        .execute(
            "INSERT INTO sync_events (provider_id, provider_name, status, message, at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![event.provider_id, event.provider_name, event.status, event.message, event.at],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

pub fn load_recent_sync_events(path: &Path, limit: usize) -> Result<Vec<StoredSyncEvent>, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare(
            "SELECT provider_id, provider_name, status, message, at FROM sync_events ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|error| error.to_string())?;

    let mapped = statement
        .query_map(params![limit as i64], |row| {
            Ok(StoredSyncEvent {
                provider_id: row.get(0)?,
                provider_name: row.get(1)?,
                status: row.get(2)?,
                message: row.get(3)?,
                at: row.get(4)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut events = Vec::new();
    for event in mapped {
        events.push(event.map_err(|error| error.to_string())?);
    }

    Ok(events)
}

pub fn upsert_provider_account(path: &Path, record: &ProviderAccountRecord) -> Result<i64, String> {
    let connection = open(path)?;
    let external = &record.external_account_id;
    connection
        .execute(
            r#"
            INSERT INTO provider_accounts (provider_id, external_account_id, display_name, email, status, first_seen_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(provider_id, external_account_id) DO UPDATE SET
              display_name = excluded.display_name,
              email = excluded.email,
              status = excluded.status,
              last_seen_at = excluded.last_seen_at
            "#,
            params![
                record.provider_id,
                external,
                record.display_name,
                record.email,
                record.status,
                record.first_seen_at,
                record.last_seen_at,
            ],
        )
        .map_err(|error| error.to_string())?;
    connection
        .query_row(
            "SELECT id FROM provider_accounts WHERE provider_id = ?1 AND external_account_id IS ?2 LIMIT 1",
            params![record.provider_id, external],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

pub fn upsert_workspace_scope(path: &Path, record: &WorkspaceScopeRecord) -> Result<i64, String> {
    let connection = open(path)?;
    let external = &record.external_scope_id;
    let parent = record.parent_scope_id;
    connection
        .execute(
            r#"
            INSERT INTO workspace_scopes (provider_account_id, scope_type, external_scope_id, parent_scope_id, display_name, is_default, first_seen_at, last_seen_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(provider_account_id, scope_type, external_scope_id) DO UPDATE SET
              parent_scope_id = excluded.parent_scope_id,
              display_name = excluded.display_name,
              is_default = excluded.is_default,
              last_seen_at = excluded.last_seen_at
            "#,
            params![
                record.provider_account_id,
                record.scope_type,
                external,
                parent,
                record.display_name,
                if record.is_default { 1 } else { 0 },
                record.first_seen_at,
                record.last_seen_at,
            ],
        )
        .map_err(|error| error.to_string())?;
    connection
        .query_row(
            "SELECT id FROM workspace_scopes WHERE provider_account_id = ?1 AND scope_type = ?2 AND external_scope_id IS ?3 LIMIT 1",
            params![record.provider_account_id, record.scope_type, external],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

pub fn load_current_provider_source_state(
    path: &Path,
    provider_id: &str,
) -> Result<Option<ProviderSourceState>, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT cs.source_kind, ss.confidence, ss.usage_access, cs.last_success_at
            FROM subscription_states ss
            JOIN connection_sources cs ON cs.id = ss.connection_source_id
            WHERE cs.provider_id = ?1 AND cs.is_enabled = 1 AND ss.is_current = 1
            ORDER BY
              CASE cs.source_kind
                WHEN 'official_api' THEN 0
                WHEN 'experimental_local_oauth' THEN 1
                WHEN 'manual' THEN 2
                ELSE 3
              END,
              ss.detected_at DESC
            LIMIT 1
            "#,
        )
        .map_err(|error| error.to_string())?;

    let mut rows = statement
        .query_map(params![provider_id], |row| {
            Ok(ProviderSourceState {
                source_kind: row.get(0)?,
                confidence: row.get(1)?,
                usage_access: row.get::<_, i64>(2)? > 0,
                last_success_at: row.get(3)?,
            })
        })
        .map_err(|error| error.to_string())?;

    match rows.next() {
        Some(row) => Ok(Some(row.map_err(|error| error.to_string())?)),
        None => Ok(None),
    }
}

pub fn upsert_connection_source(path: &Path, record: &ConnectionSourceRecord) -> Result<i64, String> {
    let connection = open(path)?;
    let cred = &record.credential_ref;
    let label = &record.source_label;
    let last_val = &record.last_validated_at;
    let last_err = &record.last_error;
    let last_succ = &record.last_success_at;
    connection
        .execute(
            r#"
            INSERT INTO connection_sources (provider_id, source_kind, credential_ref, source_label, is_enabled, last_validated_at, last_error, last_success_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(provider_id, source_kind, credential_ref) DO UPDATE SET
              source_label = excluded.source_label,
              is_enabled = excluded.is_enabled,
              last_validated_at = excluded.last_validated_at,
              last_error = excluded.last_error,
              last_success_at = excluded.last_success_at
            "#,
            params![
                record.provider_id,
                record.source_kind,
                cred,
                label,
                if record.is_enabled { 1 } else { 0 },
                last_val,
                last_err,
                last_succ,
            ],
        )
        .map_err(|error| error.to_string())?;
    connection
        .query_row(
            "SELECT id FROM connection_sources WHERE provider_id = ?1 AND source_kind = ?2 AND credential_ref IS ?3 LIMIT 1",
            params![record.provider_id, record.source_kind, cred],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

pub fn load_connection_sources(
    path: &Path,
    provider_id: &str,
) -> Result<Vec<ConnectionSourceRecord>, String> {
    let connection = open(path)?;
    let mut statement = connection
        .prepare(
            "SELECT id, provider_id, source_kind, credential_ref, source_label, is_enabled, last_validated_at, last_error, last_success_at FROM connection_sources WHERE provider_id = ?1 ORDER BY id",
        )
        .map_err(|error| error.to_string())?;
    let mapped = statement
        .query_map(params![provider_id], |row| {
            Ok(ConnectionSourceRecord {
                id: Some(row.get(0)?),
                provider_id: row.get(1)?,
                source_kind: row.get(2)?,
                credential_ref: row.get(3)?,
                source_label: row.get(4)?,
                is_enabled: row.get::<_, i64>(5)? > 0,
                last_validated_at: row.get(6)?,
                last_error: row.get(7)?,
                last_success_at: row.get(8)?,
            })
        })
        .map_err(|error| error.to_string())?;
    let mut records = Vec::new();
    for r in mapped {
        records.push(r.map_err(|error| error.to_string())?);
    }
    Ok(records)
}

pub fn upsert_subscription_state(path: &Path, record: &SubscriptionStateRecord) -> Result<i64, String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            INSERT INTO subscription_states (workspace_scope_id, connection_source_id, plan_code, plan_label, billing_status, usage_access, detected_at, effective_from, effective_to, is_current, confidence)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(workspace_scope_id, connection_source_id, detected_at) DO UPDATE SET
              plan_code = excluded.plan_code,
              plan_label = excluded.plan_label,
              billing_status = excluded.billing_status,
              usage_access = excluded.usage_access,
              effective_to = excluded.effective_to,
              is_current = excluded.is_current,
              confidence = excluded.confidence
            "#,
            params![
                record.workspace_scope_id,
                record.connection_source_id,
                record.plan_code,
                record.plan_label,
                record.billing_status,
                if record.usage_access { 1 } else { 0 },
                record.detected_at,
                record.effective_from,
                record.effective_to,
                if record.is_current { 1 } else { 0 },
                record.confidence,
            ],
        )
        .map_err(|error| error.to_string())?;
    connection
        .query_row(
            "SELECT id FROM subscription_states WHERE workspace_scope_id = ?1 AND connection_source_id = ?2 AND detected_at = ?3 LIMIT 1",
            params![
                record.workspace_scope_id,
                record.connection_source_id,
                record.detected_at,
            ],
            |row| row.get(0),
        )
        .map_err(|error| error.to_string())
}

pub fn upsert_entitlements(path: &Path, records: &[EntitlementRecord]) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }
    let mut connection = open(path)?;
    let tx = connection.transaction().map_err(|e| e.to_string())?;
    for record in records {
        tx.execute(
            r#"
            INSERT INTO entitlements (subscription_state_id, entitlement_kind, metric_key, used_value, limit_value, unit, window_seconds, resets_at, raw_label)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(subscription_state_id, entitlement_kind, metric_key) DO UPDATE SET
              used_value = excluded.used_value,
              limit_value = excluded.limit_value,
              unit = excluded.unit,
              window_seconds = excluded.window_seconds,
              resets_at = excluded.resets_at,
              raw_label = excluded.raw_label
            "#,
            params![
                record.subscription_state_id,
                record.entitlement_kind,
                record.metric_key,
                record.used_value,
                record.limit_value,
                record.unit,
                record.window_seconds,
                record.resets_at,
                record.raw_label,
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())
}

pub fn insert_usage_snapshots_v2(path: &Path, records: &[UsageSnapshotV2Record]) -> Result<(), String> {
    if records.is_empty() {
        return Ok(());
    }
    let mut connection = open(path)?;
    let tx = connection.transaction().map_err(|e| e.to_string())?;
    for record in records {
        tx.execute(
            r#"
            INSERT INTO usage_snapshots_v2 (workspace_scope_id, connection_source_id, usage_date, model_key, input_tokens, output_tokens, cached_tokens, total_tokens, request_count, cost_usd, quota_used, quota_limit, confidence)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(workspace_scope_id, connection_source_id, usage_date, model_key) DO UPDATE SET
              input_tokens = excluded.input_tokens,
              output_tokens = excluded.output_tokens,
              cached_tokens = excluded.cached_tokens,
              total_tokens = excluded.total_tokens,
              request_count = excluded.request_count,
              cost_usd = excluded.cost_usd,
              quota_used = excluded.quota_used,
              quota_limit = excluded.quota_limit,
              confidence = excluded.confidence
            "#,
            params![
                record.workspace_scope_id,
                record.connection_source_id,
                record.usage_date,
                record.model_key,
                record.input_tokens,
                record.output_tokens,
                record.cached_tokens,
                record.total_tokens,
                record.request_count,
                record.cost_usd,
                record.quota_used,
                record.quota_limit,
                record.confidence,
            ],
        )
        .map_err(|error| error.to_string())?;
    }
    tx.commit().map_err(|error| error.to_string())
}

pub fn insert_sync_run(path: &Path, record: &SyncRunRecord) -> Result<i64, String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            INSERT INTO sync_runs (connection_source_id, provider_id, status, started_at, finished_at, message, discovered_accounts, discovered_scopes, wrote_snapshots)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                record.connection_source_id,
                record.provider_id,
                record.status,
                record.started_at,
                record.finished_at,
                record.message,
                record.discovered_accounts,
                record.discovered_scopes,
                record.wrote_snapshots,
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(connection.last_insert_rowid())
}

pub fn insert_raw_observation(path: &Path, record: &RawObservationRecord) -> Result<i64, String> {
    let connection = open(path)?;
    connection
        .execute(
            r#"
            INSERT INTO raw_observations (connection_source_id, observation_kind, captured_at, payload_json, redacted)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                record.connection_source_id,
                record.observation_kind,
                record.captured_at,
                record.payload_json,
                if record.redacted { 1 } else { 0 },
            ],
        )
        .map_err(|error| error.to_string())?;
    Ok(connection.last_insert_rowid())
}

#[allow(dead_code)]
pub fn reconcile_subscriptions(path: &Path, workspace_scope_id: i64) -> Result<(), String> {
    let mut connection = open(path)?;
    let tx = connection.transaction().map_err(|e| e.to_string())?;
    tx.execute(
        "UPDATE subscription_states SET is_current = 0 WHERE workspace_scope_id = ?1",
        params![workspace_scope_id],
    )
    .map_err(|e| e.to_string())?;
    tx.execute(
        r#"
        UPDATE subscription_states SET is_current = 1
        WHERE id = (
          SELECT id FROM subscription_states
          WHERE workspace_scope_id = ?1 AND is_current = 0
          ORDER BY
            CASE confidence
              WHEN 'high' THEN 0
              WHEN 'medium' THEN 1
              WHEN 'low' THEN 2
              ELSE 3
            END,
            detected_at DESC
          LIMIT 1
        )
        "#,
        params![workspace_scope_id],
    )
    .map_err(|e| e.to_string())?;
    tx.commit().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("ai-tracker-{name}-{nanos}.sqlite"))
    }

    #[test]
    fn upsert_connection_source_returns_stable_id_on_conflict() {
        let db = temp_db_path("source-upsert");
        init_database(&db).expect("init db");

        let first = ConnectionSourceRecord {
            id: None,
            provider_id: "anthropic".to_string(),
            source_kind: "experimental_local_oauth".to_string(),
            credential_ref: Some("local_claude_code".to_string()),
            source_label: Some("Claude Code (Local)".to_string()),
            is_enabled: true,
            last_validated_at: Some("2026-01-01T00:00:00Z".to_string()),
            last_error: None,
            last_success_at: Some("2026-01-01T00:00:00Z".to_string()),
        };
        let id1 = upsert_connection_source(&db, &first).expect("insert source");

        let mut second = first.clone();
        second.last_error = Some("temporary error".to_string());
        let id2 = upsert_connection_source(&db, &second).expect("update source");

        assert_eq!(id1, id2);

        let rows = load_connection_sources(&db, "anthropic").expect("load sources");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, Some(id1));
        assert_eq!(rows[0].last_error.as_deref(), Some("temporary error"));

        let _ = std::fs::remove_file(db);
    }

    #[test]
    fn upsert_chain_returns_stable_ids() {
        let db = temp_db_path("chain-upsert");
        init_database(&db).expect("init db");

        let account = ProviderAccountRecord {
            id: None,
            provider_id: "anthropic".to_string(),
            external_account_id: Some("acct_1".to_string()),
            display_name: "Claude Account".to_string(),
            email: None,
            status: "active".to_string(),
            first_seen_at: "2026-01-01T00:00:00Z".to_string(),
            last_seen_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let account_id_1 = upsert_provider_account(&db, &account).expect("insert account");
        let account_id_2 = upsert_provider_account(&db, &account).expect("update account");
        assert_eq!(account_id_1, account_id_2);

        let scope = WorkspaceScopeRecord {
            id: None,
            provider_account_id: account_id_1,
            scope_type: "personal".to_string(),
            external_scope_id: Some("scope_1".to_string()),
            parent_scope_id: None,
            display_name: "Personal".to_string(),
            is_default: true,
            first_seen_at: "2026-01-01T00:00:00Z".to_string(),
            last_seen_at: "2026-01-01T00:00:00Z".to_string(),
        };
        let scope_id_1 = upsert_workspace_scope(&db, &scope).expect("insert scope");
        let scope_id_2 = upsert_workspace_scope(&db, &scope).expect("update scope");
        assert_eq!(scope_id_1, scope_id_2);

        let source = ConnectionSourceRecord {
            id: None,
            provider_id: "anthropic".to_string(),
            source_kind: "experimental_local_oauth".to_string(),
            credential_ref: Some("local_claude_code".to_string()),
            source_label: Some("Claude Code (Local)".to_string()),
            is_enabled: true,
            last_validated_at: None,
            last_error: None,
            last_success_at: Some("2026-01-01T00:00:00Z".to_string()),
        };
        let source_id = upsert_connection_source(&db, &source).expect("insert source");

        let state = SubscriptionStateRecord {
            id: None,
            workspace_scope_id: scope_id_1,
            connection_source_id: source_id,
            plan_code: Some("pro".to_string()),
            plan_label: Some("Pro".to_string()),
            billing_status: "active".to_string(),
            usage_access: true,
            detected_at: "2026-01-01T00:00:00Z".to_string(),
            effective_from: None,
            effective_to: None,
            is_current: true,
            confidence: "high".to_string(),
        };
        let state_id_1 = upsert_subscription_state(&db, &state).expect("insert state");
        let state_id_2 = upsert_subscription_state(&db, &state).expect("update state");
        assert_eq!(state_id_1, state_id_2);

        let _ = std::fs::remove_file(db);
    }
}
