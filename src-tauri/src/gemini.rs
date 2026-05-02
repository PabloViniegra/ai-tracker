use crate::storage::StoredUsageSnapshot;
use reqwest::Client;

const GEMINI_API_BASE: &str = "https://generativelanguage.googleapis.com/v1beta";

async fn ensure_success(response: reqwest::Response) -> Result<reqwest::Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
        return Err(
            "Gemini rechazo la API key. Verifica que la clave de AI Studio siga activa."
                .to_string(),
        );
    }

    let body = response.text().await.unwrap_or_default();
    Err(format!("Gemini respondio {status}: {body}"))
}

pub async fn validate_credentials(client: &Client, api_key: &str) -> Result<(), String> {
    let response = client
        .get(format!("{GEMINI_API_BASE}/models"))
        .query(&[("key", api_key)])
        .send()
        .await
        .map_err(|error| error.to_string())?;

    ensure_success(response).await?;
    Ok(())
}

pub async fn sync_usage(
    _client: &Client,
    _api_key: &str,
) -> Result<Vec<StoredUsageSnapshot>, String> {
    Err("Gemini Developer API valida claves de AI Studio, pero no expone un endpoint publico para consultar gasto historico ni limites restantes. Para datos reales hay que capturar usageMetadata de llamadas hechas por la app o agregar Google Cloud Billing mas adelante.".to_string())
}
