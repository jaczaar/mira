pub mod client;
pub mod error;
pub mod token_store;
pub mod types;

use std::collections::HashMap;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio::time::{timeout, Duration};
use uuid::Uuid;

use crate::config::{get_config, ConfigError};

use client::GoogleClient;
use error::GoogleError;
use token_store::{
    delete_token_for, list_accounts, load_legacy_token, load_token_for, save_token_for,
    delete_legacy_token,
};
use types::{
    AuthStartResponse, CalendarEvent, CalendarInfo, CreateEventRequest, GoogleAccountInfo,
    GoogleToken, UpdateEventRequest,
};

#[derive(Default)]
pub struct AuthState {
    session: Mutex<Option<AuthSession>>,
}

struct AuthSession {
    receiver: oneshot::Receiver<Result<GoogleAccountInfo, GoogleError>>,
}

impl From<ConfigError> for GoogleError {
    fn from(error: ConfigError) -> Self {
        GoogleError::Config(error.to_string())
    }
}

#[tauri::command]
pub async fn google_auth_start(state: tauri::State<'_, AuthState>) -> Result<AuthStartResponse, GoogleError> {
    let config = get_config()?;
    if config.google_client_id.is_empty() {
        return Err(GoogleError::Config(
            "Google Client ID is not configured".to_string(),
        ));
    }

    let client_id = config.google_client_id;
    let client_secret = if config.google_client_secret.is_empty() {
        None
    } else {
        Some(config.google_client_secret)
    };
    let (auth_url, receiver) = start_auth_flow(client_id, client_secret).await?;

    let mut session = state.session.lock().await;
    *session = Some(AuthSession { receiver });

    Ok(AuthStartResponse { auth_url })
}

#[tauri::command]
pub async fn google_auth_wait(state: tauri::State<'_, AuthState>) -> Result<GoogleAccountInfo, GoogleError> {
    let receiver = {
        let mut session = state.session.lock().await;
        session
            .take()
            .ok_or_else(|| GoogleError::Auth("No Google auth session in progress".to_string()))?
            .receiver
    };

    let result = timeout(Duration::from_secs(180), receiver)
        .await
        .map_err(|_| GoogleError::Auth("Google authentication timed out".to_string()))?;

    let account = result
        .map_err(|_| GoogleError::Auth("Google authentication canceled".to_string()))??;

    Ok(account)
}

/// Returns all connected accounts with their info
#[tauri::command]
pub async fn google_auth_status() -> Result<Vec<GoogleAccountInfo>, GoogleError> {
    let config = get_config()?;
    if config.google_client_id.is_empty() {
        return Ok(vec![]);
    }

    let client = build_client()?;

    // Migrate legacy single-token if it exists
    if let Some(legacy_token) = load_legacy_token()? {
        let token = ensure_valid_token(&client, legacy_token).await?;
        if let Ok(account) = client.get_user_info(&token.access_token).await {
            save_token_for(&account.email, &token)?;
            let _ = delete_legacy_token();
        }
    }

    let emails = list_accounts()?;
    let mut accounts = Vec::new();
    for email in emails {
        if let Some(token) = load_token_for(&email)? {
            match ensure_valid_token(&client, token).await {
                Ok(valid_token) => {
                    match client.get_user_info(&valid_token.access_token).await {
                        Ok(account) => {
                            // Re-save in case token was refreshed
                            let _ = save_token_for(&account.email, &valid_token);
                            accounts.push(account);
                        }
                        Err(_) => {} // skip accounts with invalid tokens
                    }
                }
                Err(_) => {} // skip accounts with expired/invalid tokens
            }
        }
    }

    Ok(accounts)
}

#[tauri::command]
pub async fn google_auth_sign_out(account_email: String) -> Result<(), GoogleError> {
    delete_token_for(&account_email)
}

#[tauri::command]
pub async fn google_list_calendars(account_email: String) -> Result<Vec<CalendarInfo>, GoogleError> {
    let client = build_client()?;
    let token = get_access_token_for(&client, &account_email).await?;
    client.list_calendars(&token.access_token).await
}

#[tauri::command]
pub async fn google_list_events(
    account_email: String,
    calendar_name: String,
    start_date: String,
    end_date: String,
    search_text: Option<String>,
) -> Result<Vec<CalendarEvent>, GoogleError> {
    let client = build_client()?;
    let token = get_access_token_for(&client, &account_email).await?;
    client
        .list_events(
            &token.access_token,
            &calendar_name,
            &start_date,
            &end_date,
            search_text.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn google_create_event(account_email: String, request: CreateEventRequest) -> Result<String, GoogleError> {
    let client = build_client()?;
    let token = get_access_token_for(&client, &account_email).await?;
    client.create_event(&token.access_token, request).await
}

#[tauri::command]
pub async fn google_update_event(account_email: String, request: UpdateEventRequest) -> Result<(), GoogleError> {
    let client = build_client()?;
    let token = get_access_token_for(&client, &account_email).await?;
    client.update_event(&token.access_token, request).await
}

#[tauri::command]
pub async fn google_delete_event(account_email: String, uid: String, calendar_name: String) -> Result<(), GoogleError> {
    let client = build_client()?;
    let token = get_access_token_for(&client, &account_email).await?;
    client
        .delete_event(&token.access_token, &calendar_name, &uid)
        .await
}

fn build_client() -> Result<GoogleClient, GoogleError> {
    let config = get_config()?;
    if config.google_client_id.is_empty() {
        return Err(GoogleError::Config(
            "Google Client ID is not configured".to_string(),
        ));
    }
    let client_secret = if config.google_client_secret.is_empty() {
        None
    } else {
        Some(config.google_client_secret)
    };
    GoogleClient::new(config.google_client_id, client_secret)
}

async fn get_access_token_for(client: &GoogleClient, email: &str) -> Result<GoogleToken, GoogleError> {
    let token = load_token_for(email)?.ok_or_else(|| {
        GoogleError::Auth(format!("Google account '{}' is not connected", email))
    })?;
    let valid = ensure_valid_token(client, token).await?;
    let _ = save_token_for(email, &valid);
    Ok(valid)
}

async fn ensure_valid_token(
    client: &GoogleClient,
    token: GoogleToken,
) -> Result<GoogleToken, GoogleError> {
    let now = chrono::Utc::now().timestamp();
    if token.expires_at > now + 60 {
        return Ok(token);
    }

    let refresh_token = token
        .refresh_token
        .clone()
        .ok_or_else(|| GoogleError::Auth("Missing Google refresh token".to_string()))?;
    let refreshed = client.refresh_token(&refresh_token).await?;
    Ok(refreshed)
}

async fn start_auth_flow(
    client_id: String,
    client_secret: Option<String>,
) -> Result<(String, oneshot::Receiver<Result<GoogleAccountInfo, GoogleError>>), GoogleError> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .map_err(|e| GoogleError::Io(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| GoogleError::Io(e.to_string()))?
        .port();

    let redirect_uri = format!("http://127.0.0.1:{}/oauth2/callback", port);
    let state = Uuid::new_v4().to_string();
    let code_verifier = generate_code_verifier();
    let code_challenge = code_challenge(&code_verifier);
    let auth_url = build_auth_url(&client_id, &redirect_uri, &state, &code_challenge);

    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        let result = handle_auth_callback(
            listener,
            client_id,
            client_secret,
            code_verifier,
            redirect_uri,
            state,
        )
        .await;
        let _ = sender.send(result);
    });

    Ok((auth_url, receiver))
}

async fn handle_auth_callback(
    listener: TcpListener,
    client_id: String,
    client_secret: Option<String>,
    code_verifier: String,
    redirect_uri: String,
    expected_state: String,
) -> Result<GoogleAccountInfo, GoogleError> {
    let (mut socket, _) = listener
        .accept()
        .await
        .map_err(|e| GoogleError::Io(e.to_string()))?;

    let mut buffer = [0u8; 4096];
    let bytes_read = socket
        .read(&mut buffer)
        .await
        .map_err(|e| GoogleError::Io(e.to_string()))?;

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let request_line = request.lines().next().unwrap_or_default();
    let path = request_line
        .split_whitespace()
        .nth(1)
        .unwrap_or_default();

    let query = parse_query_params(path);
    let error = query.get("error").cloned();
    let code = query.get("code").cloned();
    let state = query.get("state").cloned();

    send_auth_response(&mut socket).await?;

    if let Some(err) = error {
        return Err(GoogleError::Auth(err));
    }

    let code = code.ok_or_else(|| GoogleError::Auth("Missing authorization code".to_string()))?;
    let state = state.ok_or_else(|| GoogleError::Auth("Missing state".to_string()))?;
    if state != expected_state {
        return Err(GoogleError::Auth("State mismatch".to_string()));
    }

    let client = GoogleClient::new(client_id, client_secret)?;
    let token = client
        .exchange_code(&code, &code_verifier, &redirect_uri)
        .await?;

    let account = client.get_user_info(&token.access_token).await?;
    save_token_for(&account.email, &token)?;

    Ok(account)
}

async fn send_auth_response(socket: &mut tokio::net::TcpStream) -> Result<(), GoogleError> {
    let body = r#"
        <html>
          <head><title>Mira</title></head>
          <body>
            <h2>Google Calendar Connected</h2>
            <p>You can close this window and return to the app.</p>
          </body>
        </html>
    "#;
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    socket
        .write_all(response.as_bytes())
        .await
        .map_err(|e| GoogleError::Io(e.to_string()))?;
    Ok(())
}

fn build_auth_url(client_id: &str, redirect_uri: &str, state: &str, code_challenge: &str) -> String {
    let scope = [
        "https://www.googleapis.com/auth/calendar",
        "https://www.googleapis.com/auth/userinfo.email",
        "https://www.googleapis.com/auth/userinfo.profile",
        "openid",
    ]
    .join(" ");

    format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256&access_type=offline&prompt=consent",
        urlencoding::encode(client_id),
        urlencoding::encode(redirect_uri),
        urlencoding::encode(&scope),
        urlencoding::encode(state),
        urlencoding::encode(code_challenge)
    )
}

fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

fn code_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

fn parse_query_params(path: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let query = match path.split('?').nth(1) {
        Some(q) => q,
        None => return params,
    };

    for pair in query.split('&') {
        if pair.is_empty() {
            continue;
        }
        let mut parts = pair.splitn(2, '=');
        let key = parts.next().unwrap_or_default();
        let value = parts.next().unwrap_or_default();
        let decoded_key = urlencoding::decode(key)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| key.to_string());
        let decoded_value = urlencoding::decode(value)
            .map(|s| s.to_string())
            .unwrap_or_else(|_| value.to_string());
        params.insert(decoded_key, decoded_value);
    }

    params
}
