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

pub fn save_anthropic_settings(path: &Path, settings: &AnthropicSettingsRecord) -> Result<(), String> {
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
    let transaction = connection.transaction().map_err(|error| error.to_string())?;

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
