use crate::storage::{OpenAiSettingsRecord, StoredUsageSnapshot};
use chrono::{Duration, TimeZone, Utc};
use reqwest::{header, Client};
use serde::Deserialize;
use std::collections::HashMap;

const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

#[derive(Deserialize)]
struct UsagePage {
    data: Vec<UsageBucket>,
}

#[derive(Deserialize)]
struct UsageBucket {
    start_time: i64,
    results: Vec<UsageResult>,
}

#[derive(Deserialize)]
struct UsageResult {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    input_cached_tokens: Option<u64>,
    num_model_requests: Option<u64>,
}

#[derive(Deserialize)]
struct CostPage {
    data: Vec<CostBucket>,
}

#[derive(Deserialize)]
struct CostBucket {
    start_time: i64,
    results: Vec<CostResult>,
}

#[derive(Deserialize)]
struct CostResult {
    amount: CostAmount,
}

#[derive(Deserialize)]
struct CostAmount {
    value: f64,
}

fn request_headers(settings: &OpenAiSettingsRecord, api_key: &str) -> Result<header::HeaderMap, String> {
    let mut headers = header::HeaderMap::new();
    let auth_value = format!("Bearer {api_key}");
    headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&auth_value).map_err(|error| error.to_string())?,
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );

    if let Some(organization_id) = &settings.organization_id {
        headers.insert(
            "OpenAI-Organization",
            header::HeaderValue::from_str(organization_id).map_err(|error| error.to_string())?,
        );
    }

    if let Some(project_id) = &settings.project_id {
        headers.insert(
            "OpenAI-Project",
            header::HeaderValue::from_str(project_id).map_err(|error| error.to_string())?,
        );
    }

    Ok(headers)
}

async fn ensure_success(response: reqwest::Response, admin_message: &str) -> Result<reqwest::Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("OpenAI rechazo la API key. Verifica que la clave siga activa y pertenezca a la cuenta correcta.".to_string());
    }

    if status == reqwest::StatusCode::FORBIDDEN {
        return Err(admin_message.to_string());
    }

    Err(format!("OpenAI respondio {status}: {body}"))
}

pub async fn validate_credentials(
    client: &Client,
    settings: &OpenAiSettingsRecord,
    api_key: &str,
) -> Result<(), String> {
    let headers = request_headers(settings, api_key)?;
    let response = client
        .get(format!("{OPENAI_API_BASE}/models"))
        .headers(headers)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    ensure_success(
        response,
        "La clave autentica, pero la cuenta no puede consultar OpenAI en este contexto.",
    )
    .await?;

    Ok(())
}

pub async fn sync_usage(
    client: &Client,
    settings: &OpenAiSettingsRecord,
    api_key: &str,
) -> Result<Vec<StoredUsageSnapshot>, String> {
    let headers = request_headers(settings, api_key)?;
    let start = Utc::now().date_naive() - Duration::days(6);
    let start_time = Utc
        .from_utc_datetime(&start.and_hms_opt(0, 0, 0).ok_or("No se pudo calcular el rango inicial")?)
        .timestamp();

    let usage_response = client
        .get(format!("{OPENAI_API_BASE}/organization/usage/completions"))
        .headers(headers.clone())
        .query(&[("start_time", start_time.to_string()), ("limit", "7".to_string())])
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let usage_response = ensure_success(
        usage_response,
        "La clave autentica, pero OpenAI reserva /organization/usage/completions para Admin Keys de organizacion. Guarda la clave y usa una Admin Key para obtener consumo real.",
    )
    .await?;
    let usage_page = usage_response.json::<UsagePage>().await.map_err(|error| error.to_string())?;

    let costs_response = client
        .get(format!("{OPENAI_API_BASE}/organization/costs"))
        .headers(headers)
        .query(&[("start_time", start_time.to_string()), ("limit", "7".to_string())])
        .send()
        .await
        .map_err(|error| error.to_string())?;

    let costs_response = ensure_success(
        costs_response,
        "La clave autentica, pero OpenAI reserva /organization/costs para Admin Keys de organizacion. Guarda la clave y usa una Admin Key para obtener costos reales.",
    )
    .await?;
    let costs_page = costs_response.json::<CostPage>().await.map_err(|error| error.to_string())?;

    let mut costs_by_bucket = HashMap::new();
    for bucket in costs_page.data {
        let total_cost = bucket.results.iter().fold(0.0, |sum, result| sum + result.amount.value);
        costs_by_bucket.insert(bucket.start_time, total_cost);
    }

    let mut snapshots = Vec::new();
    for bucket in usage_page.data {
        let input_tokens = bucket
            .results
            .iter()
            .fold(0_u64, |sum, result| sum + result.input_tokens.unwrap_or(0));
        let output_tokens = bucket
            .results
            .iter()
            .fold(0_u64, |sum, result| sum + result.output_tokens.unwrap_or(0));
        let cached_tokens = bucket
            .results
            .iter()
            .fold(0_u64, |sum, result| sum + result.input_cached_tokens.unwrap_or(0));
        let request_count = bucket
            .results
            .iter()
            .fold(0_u64, |sum, result| sum + result.num_model_requests.unwrap_or(0));
        let total_tokens = input_tokens + output_tokens + cached_tokens;
        let usage_date = Utc
            .timestamp_opt(bucket.start_time, 0)
            .single()
            .ok_or("No se pudo parsear la fecha de OpenAI")?
            .date_naive()
            .to_string();

        snapshots.push(StoredUsageSnapshot {
            usage_date,
            input_tokens,
            output_tokens,
            cached_tokens,
            total_tokens,
            request_count,
            cost_usd: costs_by_bucket.get(&bucket.start_time).copied(),
            quota_used: 0,
            quota_limit: None,
        });
    }

    snapshots.sort_by(|left, right| left.usage_date.cmp(&right.usage_date));
    Ok(snapshots)
}
