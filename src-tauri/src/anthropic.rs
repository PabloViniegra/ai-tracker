use crate::storage::StoredUsageSnapshot;
use reqwest::{header, Client};

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";

fn request_headers(api_key: &str) -> Result<header::HeaderMap, String> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "x-api-key",
        header::HeaderValue::from_str(api_key).map_err(|error| error.to_string())?,
    );
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "anthropic-version",
        header::HeaderValue::from_static("2023-06-01"),
    );
    Ok(headers)
}

async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        return Err("Anthropic rechazo la API key. Verifica que la clave siga activa.".to_string());
    }

    Err(format!("Anthropic respondio {status}: {body}"))
}

pub async fn validate_credentials(client: &Client, api_key: &str) -> Result<(), String> {
    let headers = request_headers(api_key)?;
    let response = client
        .get(format!("{ANTHROPIC_API_BASE}/models"))
        .headers(headers)
        .send()
        .await
        .map_err(|error| error.to_string())?;

    ensure_success(response).await?;
    Ok(())
}

pub async fn sync_usage(_client: &Client, _api_key: &str) -> Result<Vec<StoredUsageSnapshot>, String> {
    Err("Anthropic no expone un endpoint publico de usage/costs para cuentas normales. Solo cuentas enterprise con acceso a la API de administracion pueden obtener consumo programmaticamente.".to_string())
}
