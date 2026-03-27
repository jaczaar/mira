use chrono::{DateTime, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Map, Value};

use super::error::GoogleError;
use super::types::{CalendarEvent, CalendarInfo, CreateEventRequest, GoogleAccountInfo, GoogleToken, UpdateEventRequest};

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const USERINFO_URL: &str = "https://www.googleapis.com/oauth2/v2/userinfo";
const CALENDAR_BASE: &str = "https://www.googleapis.com/calendar/v3";

#[derive(Debug)]
pub struct GoogleClient {
    client: Client,
    client_id: String,
    client_secret: Option<String>,
}

impl GoogleClient {
    pub fn new(client_id: String, client_secret: Option<String>) -> Result<Self, GoogleError> {
        let client = Client::builder()
            .build()
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        Ok(Self {
            client,
            client_id,
            client_secret,
        })
    }

    pub async fn exchange_code(
        &self,
        code: &str,
        code_verifier: &str,
        redirect_uri: &str,
    ) -> Result<GoogleToken, GoogleError> {
        let mut params = vec![
            ("client_id", self.client_id.clone()),
            ("code", code.to_string()),
            ("code_verifier", code_verifier.to_string()),
            ("grant_type", "authorization_code".to_string()),
            ("redirect_uri", redirect_uri.to_string()),
        ];

        if let Some(secret) = &self.client_secret {
            params.push(("client_secret", secret.clone()));
        }

        self.request_token(params).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<GoogleToken, GoogleError> {
        let mut params = vec![
            ("client_id", self.client_id.clone()),
            ("refresh_token", refresh_token.to_string()),
            ("grant_type", "refresh_token".to_string()),
        ];

        if let Some(secret) = &self.client_secret {
            params.push(("client_secret", secret.clone()));
        }

        let mut token = self.request_token(params).await?;
        token.refresh_token = Some(refresh_token.to_string());
        Ok(token)
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<GoogleAccountInfo, GoogleError> {
        let response = self
            .client
            .get(USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        if !status.is_success() {
            return Err(GoogleError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        #[derive(Deserialize)]
        struct UserInfoResponse {
            email: String,
            name: Option<String>,
        }

        let info: UserInfoResponse =
            serde_json::from_str(&body).map_err(|e| GoogleError::Parse(e.to_string()))?;

        Ok(GoogleAccountInfo {
            email: info.email,
            name: info.name,
        })
    }

    pub async fn list_calendars(&self, access_token: &str) -> Result<Vec<CalendarInfo>, GoogleError> {
        let url = format!("{}/users/me/calendarList", CALENDAR_BASE);

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        if !status.is_success() {
            return Err(GoogleError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        #[derive(Deserialize)]
        struct CalendarListResponse {
            items: Vec<CalendarListItem>,
        }

        #[derive(Deserialize)]
        struct CalendarListItem {
            id: String,
            summary: Option<String>,
        }

        let parsed: CalendarListResponse =
            serde_json::from_str(&body).map_err(|e| GoogleError::Parse(e.to_string()))?;

        let calendars = parsed
            .items
            .into_iter()
            .map(|item| CalendarInfo {
                name: item.summary.unwrap_or_else(|| item.id.clone()),
                uid: item.id,
            })
            .collect();

        Ok(calendars)
    }

    pub async fn list_events(
        &self,
        access_token: &str,
        calendar_id: &str,
        start_date: &str,
        end_date: &str,
        search_text: Option<&str>,
    ) -> Result<Vec<CalendarEvent>, GoogleError> {
        let encoded_id = urlencoding::encode(calendar_id);
        let url = format!("{}/calendars/{}/events", CALENDAR_BASE, encoded_id);

        let time_min = to_rfc3339(start_date)?;
        let time_max = to_rfc3339(end_date)?;
        let mut events: Vec<CalendarEvent> = Vec::new();
        let mut page_token: Option<String> = None;

        loop {
            let mut request = self
                .client
                .get(&url)
                .bearer_auth(access_token)
                .query(&[
                    ("timeMin", time_min.as_str()),
                    ("timeMax", time_max.as_str()),
                    ("singleEvents", "true"),
                    ("orderBy", "startTime"),
                    ("maxResults", "2500"),
                ]);

            if let Some(query) = search_text {
                request = request.query(&[("q", query)]);
            }

            if let Some(token) = &page_token {
                request = request.query(&[("pageToken", token)]);
            }

            let response = request
                .send()
                .await
                .map_err(|e| GoogleError::Request(e.to_string()))?;

            let status = response.status();
            let body = response
                .text()
                .await
                .map_err(|e| GoogleError::Request(e.to_string()))?;

            if !status.is_success() {
                return Err(GoogleError::Api {
                    status: status.as_u16(),
                    message: body,
                });
            }

            #[derive(Deserialize)]
            struct EventsResponse {
                items: Vec<EventItem>,
                #[serde(rename = "nextPageToken")]
                next_page_token: Option<String>,
            }

            #[derive(Deserialize)]
            struct EventItem {
                id: String,
                summary: Option<String>,
                description: Option<String>,
                #[serde(rename = "htmlLink")]
                html_link: Option<String>,
                source: Option<EventSource>,
                start: EventDateTime,
                end: EventDateTime,
            }

            let parsed: EventsResponse =
                serde_json::from_str(&body).map_err(|e| GoogleError::Parse(e.to_string()))?;

            for event in parsed.items {
                let start = normalize_event_datetime(&event.start);
                let end = normalize_event_datetime(&event.end);
                if start.is_empty() || end.is_empty() {
                    continue;
                }

                events.push(CalendarEvent {
                    uid: event.id,
                    summary: event.summary.unwrap_or_else(|| "Untitled".to_string()),
                    start_date: start,
                    end_date: end,
                    description: event.description,
                    url: event
                        .source
                        .and_then(|source| source.url)
                        .or(event.html_link),
                    calendar_name: calendar_id.to_string(),
                });
            }

            if let Some(token) = parsed.next_page_token {
                page_token = Some(token);
            } else {
                break;
            }
        }

        Ok(events)
    }

    pub async fn create_event(
        &self,
        access_token: &str,
        request: CreateEventRequest,
    ) -> Result<String, GoogleError> {
        let encoded_id = urlencoding::encode(&request.calendar_name);
        let url = format!("{}/calendars/{}/events", CALENDAR_BASE, encoded_id);

        let payload = build_event_payload(
            request.summary,
            request.start_date,
            request.end_date,
            request.description,
            request.url,
            request.is_focus_time,
            request.color_id,
        )?;

        let response = self
            .client
            .post(&url)
            .bearer_auth(access_token)
            .json(&payload)
            .send()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        if !status.is_success() {
            return Err(GoogleError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        #[derive(Deserialize)]
        struct CreateEventResponse {
            id: String,
        }

        let parsed: CreateEventResponse =
            serde_json::from_str(&body).map_err(|e| GoogleError::Parse(e.to_string()))?;

        Ok(parsed.id)
    }

    pub async fn update_event(
        &self,
        access_token: &str,
        request: UpdateEventRequest,
    ) -> Result<(), GoogleError> {
        let encoded_id = urlencoding::encode(&request.calendar_name);
        let encoded_event = urlencoding::encode(&request.uid);
        let url = format!(
            "{}/calendars/{}/events/{}",
            CALENDAR_BASE, encoded_id, encoded_event
        );

        let payload = build_partial_event_payload(request)?;

        if payload.is_empty() {
            return Ok(());
        }

        let response = self
            .client
            .patch(&url)
            .bearer_auth(access_token)
            .json(&Value::Object(payload))
            .send()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(GoogleError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        Ok(())
    }

    pub async fn delete_event(
        &self,
        access_token: &str,
        calendar_id: &str,
        event_id: &str,
    ) -> Result<(), GoogleError> {
        let encoded_id = urlencoding::encode(calendar_id);
        let encoded_event = urlencoding::encode(event_id);
        let url = format!(
            "{}/calendars/{}/events/{}",
            CALENDAR_BASE, encoded_id, encoded_event
        );

        let response = self
            .client
            .delete(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        let status = response.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(GoogleError::Api {
                status: status.as_u16(),
                message: body,
            })
        }
    }

    async fn request_token(&self, params: Vec<(&str, String)>) -> Result<GoogleToken, GoogleError> {
        let response = self
            .client
            .post(TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| GoogleError::Request(e.to_string()))?;

        if !status.is_success() {
            return Err(GoogleError::Api {
                status: status.as_u16(),
                message: body,
            });
        }

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            expires_in: i64,
            refresh_token: Option<String>,
            token_type: String,
            scope: Option<String>,
        }

        let parsed: TokenResponse =
            serde_json::from_str(&body).map_err(|e| GoogleError::Parse(e.to_string()))?;

        Ok(GoogleToken {
            access_token: parsed.access_token,
            refresh_token: parsed.refresh_token,
            expires_at: Utc::now().timestamp() + parsed.expires_in,
            token_type: parsed.token_type,
            scope: parsed.scope,
        })
    }
}

fn build_event_payload(
    summary: String,
    start_date: String,
    end_date: String,
    description: Option<String>,
    url: Option<String>,
    is_focus_time: bool,
    color_id: Option<String>,
) -> Result<Value, GoogleError> {
    let mut map = Map::new();
    map.insert("summary".to_string(), Value::String(summary));

    if let Some(desc) = description {
        map.insert("description".to_string(), Value::String(desc));
    }

    let start = to_rfc3339(&start_date)?;
    let end = to_rfc3339(&end_date)?;
    map.insert(
        "start".to_string(),
        json!({
            "dateTime": start
        }),
    );
    map.insert(
        "end".to_string(),
        json!({
            "dateTime": end
        }),
    );

    // Add color if specified
    if let Some(color) = color_id {
        map.insert("colorId".to_string(), Value::String(color));
    }

    if is_focus_time {
        // Focus time events have restrictions - they cannot have source/attendees
        // and require specific properties
        map.insert("eventType".to_string(), Value::String("focusTime".to_string()));
        map.insert("transparency".to_string(), Value::String("opaque".to_string()));
        map.insert(
            "focusTimeProperties".to_string(),
            json!({
                "chatStatus": "doNotDisturb",
                "autoDeclineMode": "declineOnlyNewConflictingInvitations"
            }),
        );
        // Add URL to description instead of source for focus time events
        if let Some(link) = url {
            let current_desc = map.get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let new_desc = if current_desc.is_empty() {
                format!("Link: {}", link)
            } else {
                format!("{}\n\nLink: {}", current_desc, link)
            };
            map.insert("description".to_string(), Value::String(new_desc));
        }
    } else {
        // Regular events can have source
        if let Some(link) = url {
            map.insert("source".to_string(), json!({ "title": "Jira Issue", "url": link }));
        }
    }

    Ok(Value::Object(map))
}

fn build_partial_event_payload(
    request: UpdateEventRequest,
) -> Result<Map<String, Value>, GoogleError> {
    let mut map = Map::new();

    if let Some(summary) = request.summary {
        map.insert("summary".to_string(), Value::String(summary));
    }

    if let Some(description) = request.description {
        map.insert("description".to_string(), Value::String(description));
    }

    if let Some(start_date) = request.start_date {
        let start = to_rfc3339(&start_date)?;
        map.insert("start".to_string(), json!({ "dateTime": start }));
    }

    if let Some(end_date) = request.end_date {
        let end = to_rfc3339(&end_date)?;
        map.insert("end".to_string(), json!({ "dateTime": end }));
    }

    if let Some(url) = request.url {
        map.insert("source".to_string(), json!({ "title": "Jira Issue", "url": url }));
    }

    if let Some(color_id) = request.color_id {
        map.insert("colorId".to_string(), Value::String(color_id));
    }

    // Note: We don't update focus time status on existing events
    // as it requires deleting and recreating the event.
    // The is_focus_time field is ignored for updates.

    Ok(map)
}

fn to_rfc3339(date_str: &str) -> Result<String, GoogleError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.to_rfc3339());
    }

    if let Ok(dt) = NaiveDateTime::parse_from_str(date_str, "%Y-%m-%dT%H:%M:%S") {
        let local_dt = Local
            .from_local_datetime(&dt)
            .single()
            .ok_or_else(|| GoogleError::Parse("Invalid local datetime".to_string()))?;
        return Ok(local_dt.to_rfc3339());
    }

    if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        let naive = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| GoogleError::Parse("Invalid date".to_string()))?;
        let local_dt = Local
            .from_local_datetime(&naive)
            .single()
            .ok_or_else(|| GoogleError::Parse("Invalid local date".to_string()))?;
        return Ok(local_dt.to_rfc3339());
    }

    Err(GoogleError::Parse(format!(
        "Unsupported date format: {}",
        date_str
    )))
}

fn normalize_event_datetime(event: &EventDateTime) -> String {
    if let Some(dt) = &event.date_time {
        return dt.to_string();
    }
    if let Some(date) = &event.date {
        return format!("{}T00:00:00", date);
    }
    String::new()
}
#[derive(Deserialize)]
struct EventDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
}

#[derive(Deserialize)]
struct EventSource {
    url: Option<String>,
}
