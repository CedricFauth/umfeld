use std::{str::FromStr, sync::Arc};

use axum::{
    Json,
    extract::{Query, Request, State},
    http::{StatusCode, header},
    middleware::Next,
    response::{Html, IntoResponse, Response},
};
use chrono::{self, Utc};
use chrono_tz::{self, Tz};
use serde::Deserialize;
use tower_governor::GovernorError;
mod provider;
pub mod response;
mod wire;
use crate::{
    provider::{ApiProviderError, get_fuel_prices, get_weather_data, resolve_place},
    response::{Attribution, SnapshotResponse},
};

impl SnapshotResponse {
    fn from_error(err: ApiProviderError) -> SnapshotResponse {
        SnapshotResponse::from_message(err.to_string())
    }

    fn from_message(message: String) -> SnapshotResponse {
        SnapshotResponse {
            success: false,
            error: Some(message),
            datetime: Utc::now().with_timezone(&chrono_tz::UTC).to_rfc3339(),
            attribution: Vec::new(),
            place: None,
            weather: None,
            fuel: None,
        }
    }
}

#[derive(Deserialize)]
pub struct SnapshotRequest {
    place: String,
}

#[derive(Clone)]
pub struct AppState {
    pub fuel_api_key: Arc<str>,
    pub web_client: reqwest::Client,
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>umfeld</title>
</head>
<body>
<h1>umfeld</h1>
<p>Time, weather and fuel prices for a place.</p>
<p>Usage: <code>GET /snapshot?place=Dresden</code></p>
<p>Weather data by <a href="https://open-meteo.com/">Open-Meteo.com</a>, fuel prices by
<a href="https://creativecommons.tankerkoenig.de/">Tankerkönig</a> (MTS-K), both CC BY 4.0.</p>
</body>
</html>
"#;

pub async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

/// 400 if `place` is missing.
pub async fn require_place(req: Request, next: Next) -> Response {
    if let Err(rejection) = Query::<SnapshotRequest>::try_from_uri(req.uri()) {
        return (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
            Json(SnapshotResponse::from_message(rejection.body_text())),
        )
            .into_response();
    }
    next.run(req).await
}

pub async fn snapshot(
    State(state): State<AppState>,
    Query(q): Query<SnapshotRequest>,
) -> impl IntoResponse {
    let data = fetch_data(&state.web_client, &q.place, &state.fuel_api_key).await;

    (
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(data),
    )
}

async fn fetch_data(
    client: &reqwest::Client,
    location: &str,
    fuel_api_key: &str,
) -> SnapshotResponse {
    let place = match resolve_place(client, location).await {
        Ok(place) => place,
        Err(error) => return SnapshotResponse::from_error(error),
    };

    let (weather, fuel) = match tokio::try_join!(
        get_weather_data(client, &place),
        get_fuel_prices(client, &place, 7.0, fuel_api_key),
    ) {
        Ok(pair) => pair,
        Err(error) => return SnapshotResponse::from_error(error),
    };

    let datetime = match Tz::from_str(&place.timezone) {
        Ok(tz) => Utc::now().with_timezone(&tz),
        Err(err) => {
            eprintln!("error getting timezone, using UTC: {err}");
            Utc::now().with_timezone(&chrono_tz::UTC)
        }
    };

    SnapshotResponse {
        success: true,
        error: None,
        datetime: datetime.to_rfc3339(),
        attribution: vec![
            Attribution::open_meteo(),
            Attribution::tankerkoenig(fuel.license),
        ],
        place: Some(place),
        weather: Some(weather),
        fuel: Some(fuel.stations),
    }
}

pub fn rate_limit_response(err: GovernorError) -> Response {
    match err {
        GovernorError::TooManyRequests { wait_time, .. } => (
            StatusCode::TOO_MANY_REQUESTS,
            [
                (header::RETRY_AFTER, wait_time.to_string()),
                (
                    header::CONTENT_TYPE,
                    "application/json; charset=utf-8".to_string(),
                ),
            ],
            Json(SnapshotResponse::from_message(format!(
                "too many requests, wait for {wait_time}s"
            ))),
        )
            .into_response(),
        other => other.into_response().map(axum::body::Body::new),
    }
}
