use keyring::Entry;

const OPENAI_ACCOUNT: &str = "openai_api_key";
const ANTHROPIC_ACCOUNT: &str = "anthropic_api_key";

pub fn save_openai_api_key(service: &str, api_key: &str) -> Result<(), String> {
    let entry = Entry::new(service, OPENAI_ACCOUNT).map_err(|error| error.to_string())?;
    entry
        .set_password(api_key)
        .map_err(|error| error.to_string())
}

pub fn load_openai_api_key(service: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(service, OPENAI_ACCOUNT).map_err(|error| error.to_string())?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

pub fn save_anthropic_api_key(service: &str, api_key: &str) -> Result<(), String> {
    let entry = Entry::new(service, ANTHROPIC_ACCOUNT).map_err(|error| error.to_string())?;
    entry
        .set_password(api_key)
        .map_err(|error| error.to_string())
}

pub fn load_anthropic_api_key(service: &str) -> Result<Option<String>, String> {
    let entry = Entry::new(service, ANTHROPIC_ACCOUNT).map_err(|error| error.to_string())?;
    match entry.get_password() {
        Ok(secret) => Ok(Some(secret)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}
