mod anthropic;
mod connectors;
mod domain;
mod normalization;
mod reconciliation;
mod openai;
mod providers;
mod security;
mod storage;

use chrono::{Datelike, Duration, NaiveDate, Utc};
use domain::{
    AnthropicConnectionState, DashboardSnapshot, OpenAiConnectionState, ProviderId,
    SaveAnthropicCredentialsInput, SaveAnthropicCredentialsResult, SaveOpenAiCredentialsInput,
    SaveOpenAiCredentialsResult, SyncEvent, SyncEventStatus, UsagePoint,
};
use providers::{base_provider_catalog, merge_anthropic_source_info, merge_anthropic_summary, merge_openai_summary};
use reqwest::Client;
use std::fs;
use std::path::PathBuf;
use storage::{AnthropicSettingsRecord, OpenAiSettingsRecord, StoredSyncEvent, StoredUsageSnapshot};
use tauri::{Manager, State};

struct AppState {
    db_path: PathBuf,
    keyring_service: String,
    http_client: Client,
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn trim_optional(value: Option<String>) -> Option<String> {
    value.and_then(|entry| {
        let trimmed = entry.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    })
}

fn weekday_label(date: NaiveDate) -> String {
    match date.weekday().number_from_monday() {
        1 => "Lun",
        2 => "Mar",
        3 => "Mie",
        4 => "Jue",
        5 => "Vie",
        6 => "Sab",
        _ => "Dom",
    }
    .to_string()
}

fn sync_event_status_from_str(status: &str) -> SyncEventStatus {
    match status {
        "warning" => SyncEventStatus::Warning,
        "error" => SyncEventStatus::Error,
        _ => SyncEventStatus::Success,
    }
}

fn provider_id_from_str(value: &str) -> Option<ProviderId> {
    match value {
        "openai" => Some(ProviderId::Openai),
        "anthropic" => Some(ProviderId::Anthropic),
        _ => None,
    }
}

fn connection_state_from_record(record: OpenAiSettingsRecord) -> OpenAiConnectionState {
    let usage_access =
        record.has_credentials && record.last_sync_at.is_some() && record.last_error.is_none();

    OpenAiConnectionState {
        has_credentials: record.has_credentials,
        account_label: record.account_label,
        organization_id: record.organization_id,
        project_id: record.project_id,
        last_validated_at: record.last_validated_at,
        last_sync_at: record.last_sync_at,
        usage_access,
        last_error: record.last_error,
    }
}

fn anthropic_connection_state_from_record(
    record: AnthropicSettingsRecord,
) -> AnthropicConnectionState {
    let usage_access =
        record.has_credentials && record.last_sync_at.is_some() && record.last_error.is_none();

    AnthropicConnectionState {
        has_credentials: record.has_credentials,
        account_label: record.account_label,
        last_validated_at: record.last_validated_at,
        last_sync_at: record.last_sync_at,
        usage_access,
        last_error: record.last_error,
    }
}

fn history_points_from_usage(usage: &[StoredUsageSnapshot]) -> Vec<UsagePoint> {
    if !usage.is_empty() {
        return usage
            .iter()
            .filter_map(|snapshot| {
                NaiveDate::parse_from_str(&snapshot.usage_date, "%Y-%m-%d")
                    .ok()
                    .map(|date| UsagePoint {
                        day: weekday_label(date),
                        tokens: snapshot.total_tokens,
                        cost_usd: snapshot.cost_usd.unwrap_or(0.0),
                    })
            })
            .collect();
    }

    (0..7)
        .map(|offset| Utc::now().date_naive() - Duration::days(6 - offset))
        .map(|date| UsagePoint {
            day: weekday_label(date),
            tokens: 0,
            cost_usd: 0.0,
        })
        .collect()
}

fn dashboard_snapshot(state: &AppState) -> Result<DashboardSnapshot, String> {
    let openai_settings = storage::load_openai_settings(&state.db_path)?;
    let openai_usage = storage::load_usage_history(&state.db_path, ProviderId::Openai.as_str(), 7)?;
    let anthropic_settings = storage::load_anthropic_settings(&state.db_path)?;
    let anthropic_usage =
        storage::load_usage_history(&state.db_path, ProviderId::Anthropic.as_str(), 7)?;
    let events = storage::load_recent_sync_events(&state.db_path, 10)?;

    let openai_connection = connection_state_from_record(openai_settings);
    let anthropic_connection = anthropic_connection_state_from_record(anthropic_settings);

    let mut providers = base_provider_catalog();
    merge_openai_summary(&mut providers, &openai_connection, &openai_usage);
    merge_anthropic_summary(&mut providers, &anthropic_connection, &anthropic_usage);

    if let Some(source_state) =
        storage::load_current_provider_source_state(&state.db_path, "anthropic")?
    {
        merge_anthropic_source_info(&mut providers, &source_state);
    }

    let sync_events = events
        .into_iter()
        .filter_map(|event| {
            provider_id_from_str(&event.provider_id).map(|provider_id| SyncEvent {
                provider_id,
                provider_name: event.provider_name,
                status: sync_event_status_from_str(&event.status),
                message: event.message,
                at: event.at,
            })
        })
        .collect();

    let history = if !openai_usage.is_empty() {
        history_points_from_usage(&openai_usage)
    } else {
        history_points_from_usage(&anthropic_usage)
    };

    Ok(DashboardSnapshot {
        providers,
        history,
        sync_events,
    })
}

fn append_openai_event(
    state: &AppState,
    status: &str,
    message: impl Into<String>,
    at: impl Into<String>,
) -> Result<(), String> {
    storage::append_sync_event(
        &state.db_path,
        &StoredSyncEvent {
            provider_id: ProviderId::Openai.as_str().to_string(),
            provider_name: ProviderId::Openai.display_name().to_string(),
            status: status.to_string(),
            message: message.into(),
            at: at.into(),
        },
    )
}

async fn sync_openai_internal(state: &AppState) -> Result<Option<String>, String> {
    let mut settings = storage::load_openai_settings(&state.db_path)?;
    let Some(api_key) = security::load_openai_api_key(&state.keyring_service)? else {
        return Ok(Some(
            "Configura una API key de OpenAI antes de sincronizar.".to_string(),
        ));
    };

    let synced_at = now_iso();
    match openai::sync_usage(&state.http_client, &settings, &api_key).await {
        Ok(snapshots) => {
            storage::replace_usage_snapshots(
                &state.db_path,
                ProviderId::Openai.as_str(),
                &snapshots,
            )?;
            settings.last_sync_at = Some(synced_at.clone());
            settings.last_error = None;
            settings.has_credentials = true;
            storage::save_openai_settings(&state.db_path, &settings)?;
            append_openai_event(
                state,
                "success",
                "OpenAI sincronizado desde endpoints oficiales de usage/costs.",
                synced_at,
            )?;
            Ok(None)
        }
        Err(message) => {
            settings.last_error = Some(message.clone());
            settings.has_credentials = true;
            storage::save_openai_settings(&state.db_path, &settings)?;
            let status = if message.contains("Admin Key") {
                "warning"
            } else {
                "error"
            };
            append_openai_event(state, status, message.clone(), synced_at)?;
            Ok(Some(message))
        }
    }
}

fn append_anthropic_event(
    state: &AppState,
    status: &str,
    message: impl Into<String>,
    at: impl Into<String>,
) -> Result<(), String> {
    storage::append_sync_event(
        &state.db_path,
        &StoredSyncEvent {
            provider_id: ProviderId::Anthropic.as_str().to_string(),
            provider_name: ProviderId::Anthropic.display_name().to_string(),
            status: status.to_string(),
            message: message.into(),
            at: at.into(),
        },
    )
}

async fn sync_anthropic_internal(state: &AppState) -> Result<Option<String>, String> {
    let mut settings = storage::load_anthropic_settings(&state.db_path)?;
    let Some(api_key) = security::load_anthropic_api_key(&state.keyring_service)? else {
        return Ok(Some(
            "Configura una API key de Anthropic antes de sincronizar.".to_string(),
        ));
    };

    let synced_at = now_iso();
    match anthropic::sync_usage(&state.http_client, &api_key).await {
        Ok(snapshots) => {
            storage::replace_usage_snapshots(
                &state.db_path,
                ProviderId::Anthropic.as_str(),
                &snapshots,
            )?;
            settings.last_sync_at = Some(synced_at.clone());
            settings.last_error = None;
            settings.has_credentials = true;
            storage::save_anthropic_settings(&state.db_path, &settings)?;
            append_anthropic_event(
                state,
                "success",
                "Anthropic sincronizado desde endpoints oficiales.",
                synced_at,
            )?;
            Ok(None)
        }
        Err(message) => {
            settings.last_error = Some(message.clone());
            settings.has_credentials = true;
            storage::save_anthropic_settings(&state.db_path, &settings)?;
            let status = if message.contains("no expone") || message.contains("endpoint publico") {
                "warning"
            } else {
                "error"
            };
            append_anthropic_event(state, status, message.clone(), synced_at)?;
            Ok(Some(message))
        }
    }
}

#[tauri::command]
fn get_dashboard_snapshot(state: State<'_, AppState>) -> Result<DashboardSnapshot, String> {
    dashboard_snapshot(&state)
}

#[tauri::command]
fn get_openai_connection(state: State<'_, AppState>) -> Result<OpenAiConnectionState, String> {
    let record = storage::load_openai_settings(&state.db_path)?;
    Ok(connection_state_from_record(record))
}

#[tauri::command]
async fn save_openai_credentials(
    state: State<'_, AppState>,
    input: SaveOpenAiCredentialsInput,
) -> Result<SaveOpenAiCredentialsResult, String> {
    let api_key = input.api_key.trim().to_string();
    if api_key.is_empty() {
        return Err("La API key de OpenAI no puede estar vacia.".to_string());
    }

    let mut settings = storage::load_openai_settings(&state.db_path)?;
    settings.has_credentials = true;
    settings.account_label = trim_optional(input.account_label);
    settings.organization_id = trim_optional(input.organization_id);
    settings.project_id = trim_optional(input.project_id);
    settings.last_validated_at = Some(now_iso());
    settings.last_error = None;

    openai::validate_credentials(&state.http_client, &settings, &api_key).await?;
    security::save_openai_api_key(&state.keyring_service, &api_key)?;
    storage::save_openai_settings(&state.db_path, &settings)?;

    let sync_warning = sync_openai_internal(&state).await?;
    let connection = connection_state_from_record(storage::load_openai_settings(&state.db_path)?);
    let message = match sync_warning {
        Some(warning) => format!("Credenciales guardadas. {warning}"),
        None => "Credenciales guardadas y sincronizacion inicial completada.".to_string(),
    };

    Ok(SaveOpenAiCredentialsResult {
        connection,
        message,
    })
}

#[tauri::command]
fn get_anthropic_connection(
    state: State<'_, AppState>,
) -> Result<AnthropicConnectionState, String> {
    let record = storage::load_anthropic_settings(&state.db_path)?;
    Ok(anthropic_connection_state_from_record(record))
}

#[tauri::command]
async fn save_anthropic_credentials(
    state: State<'_, AppState>,
    input: SaveAnthropicCredentialsInput,
) -> Result<SaveAnthropicCredentialsResult, String> {
    let api_key = input.api_key.trim().to_string();
    if api_key.is_empty() {
        return Err("La API key de Anthropic no puede estar vacia.".to_string());
    }

    let mut settings = storage::load_anthropic_settings(&state.db_path)?;
    settings.has_credentials = true;
    settings.account_label = trim_optional(input.account_label);
    settings.last_validated_at = Some(now_iso());
    settings.last_error = None;

    anthropic::validate_credentials(&state.http_client, &api_key).await?;
    security::save_anthropic_api_key(&state.keyring_service, &api_key)?;
    storage::save_anthropic_settings(&state.db_path, &settings)?;

    let sync_warning = sync_anthropic_internal(&state).await?;
    let connection =
        anthropic_connection_state_from_record(storage::load_anthropic_settings(&state.db_path)?);
    let message = match sync_warning {
        Some(warning) => format!("Credenciales guardadas. {warning}"),
        None => "Credenciales guardadas y sincronizacion inicial completada.".to_string(),
    };

    Ok(SaveAnthropicCredentialsResult {
        connection,
        message,
    })
}

#[tauri::command]
async fn sync_all_providers(state: State<'_, AppState>) -> Result<DashboardSnapshot, String> {
    let openai_settings = storage::load_openai_settings(&state.db_path)?;
    if openai_settings.has_credentials {
        let _ = sync_openai_internal(&state).await?;
    }

    let anthropic_settings = storage::load_anthropic_settings(&state.db_path)?;
    if anthropic_settings.has_credentials {
        let _ = sync_anthropic_internal(&state).await?;
    }

    dashboard_snapshot(&state)
}

#[tauri::command]
async fn sync_anthropic_experimental(state: State<'_, AppState>) -> Result<String, String> {
    use crate::connectors::{AnthropicExperimentalConnector, ProviderConnector, SyncPackage};

    let connector = AnthropicExperimentalConnector::new(state.http_client.clone());

    let conn_source = crate::domain::ConnectionSourceRecord {
        id: None,
        provider_id: "anthropic".to_string(),
        source_kind: "experimental_local_oauth".to_string(),
        credential_ref: Some("local_claude_code".to_string()),
        source_label: Some("Claude Code (Local)".to_string()),
        is_enabled: true,
        last_validated_at: None,
        last_error: None,
        last_success_at: None,
    };
    let source_id = storage::upsert_connection_source(&state.db_path, &conn_source)?;

    let started_at = now_iso();
    let sync_result: Result<SyncPackage, crate::connectors::ConnectorError> = connector.sync("local").await;

    match sync_result {
        Ok(pkg) => {
            normalization::normalize_and_store(&state.db_path, source_id, pkg)?;

            let mut success_conn = conn_source.clone();
            success_conn.id = Some(source_id);
            success_conn.last_success_at = Some(now_iso());
            success_conn.last_error = None;
            storage::upsert_connection_source(&state.db_path, &success_conn)?;

            let sync_run = crate::domain::SyncRunRecord {
                id: None,
                connection_source_id: source_id,
                provider_id: "anthropic".to_string(),
                status: "success".to_string(),
                started_at,
                finished_at: Some(now_iso()),
                message: Some("Sync completed successfully".to_string()),
                discovered_accounts: 1,
                discovered_scopes: 1,
                wrote_snapshots: 0,
            };
            storage::insert_sync_run(&state.db_path, &sync_run)?;

            Ok("Sync completed successfully".to_string())
        }
        Err(e) => {
            let mut fail_conn = conn_source;
            fail_conn.id = Some(source_id);
            fail_conn.last_error = Some(e.to_string());
            storage::upsert_connection_source(&state.db_path, &fail_conn)?;

            let sync_run = crate::domain::SyncRunRecord {
                id: None,
                connection_source_id: source_id,
                provider_id: "anthropic".to_string(),
                status: "warning".to_string(),
                started_at,
                finished_at: Some(now_iso()),
                message: Some(e.to_string()),
                discovered_accounts: 0,
                discovered_scopes: 0,
                wrote_snapshots: 0,
            };
            storage::insert_sync_run(&state.db_path, &sync_run)?;

            Err(e.to_string())
        }
    }
}

#[tauri::command]
fn get_anthropic_sources(state: State<'_, AppState>) -> Result<Vec<crate::domain::ConnectionSourceRecord>, String> {
    storage::load_connection_sources(&state.db_path, "anthropic")
}

#[tauri::command]
async fn enable_anthropic_experimental(state: State<'_, AppState>, enabled: bool) -> Result<(), String> {
    let sources = storage::load_connection_sources(&state.db_path, "anthropic")?;
    for mut source in sources {
        if source.source_kind == "experimental_local_oauth" {
            source.is_enabled = enabled;
            storage::upsert_connection_source(&state.db_path, &source)?;
        }
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            fs::create_dir_all(&app_data_dir).map_err(|error| error.to_string())?;

            let db_path = app_data_dir.join("ai-tracker.sqlite3");
            storage::init_database(&db_path)?;

            app.manage(AppState {
                db_path,
                keyring_service: "com.pablo.ai-tracker".to_string(),
                http_client: Client::new(),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_dashboard_snapshot,
            get_openai_connection,
            save_openai_credentials,
            get_anthropic_connection,
            save_anthropic_credentials,
            sync_all_providers,
            sync_anthropic_experimental,
            get_anthropic_sources,
            enable_anthropic_experimental
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
