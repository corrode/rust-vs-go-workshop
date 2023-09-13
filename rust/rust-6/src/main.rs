use anyhow::Context;
use askama::Template;
use async_trait::async_trait;
use axum::{
    extract::{FromRequestParts, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use http::request::Parts;
use serde::Deserialize;
use sqlx::PgPool;
use std::str::from_utf8;

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[derive(Deserialize, Debug)]
pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

#[derive(sqlx::FromRow, Deserialize, Debug, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Deserialize, Debug)]
pub struct WeatherResponse {
    pub latitude: f64,
    pub longitude: f64,
    pub timezone: String,
    pub hourly: Hourly,
}

#[derive(Deserialize, Debug)]
pub struct Hourly {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
}

#[derive(Template, Deserialize, Debug)]
#[template(path = "weather.html")]
pub struct WeatherDisplay {
    pub city: String,
    pub forecasts: Vec<Forecast>,
}

#[derive(Deserialize, Debug)]
pub struct Forecast {
    pub date: String,
    pub temperature: String,
}

#[derive(sqlx::FromRow, Deserialize, Debug)]
pub struct City {
    pub name: String,
}

async fn get_last_cities(pool: &PgPool) -> Result<Vec<City>, anyhow::Error> {
    let cities = sqlx::query_as::<_, City>("SELECT name FROM cities ORDER BY id DESC LIMIT 10")
        .fetch_all(pool)
        .await?;
    Ok(cities)
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, anyhow::Error> {
    let endpoint = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
    response.results.get(0).cloned().context("No results found")
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, anyhow::Error> {
    let endpoint = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&hourly=temperature_2m",
        lat_long.latitude, lat_long.longitude
    );
    let response = reqwest::get(&endpoint)
        .await?
        .json::<WeatherResponse>()
        .await?;
    Ok(response)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn index() -> IndexTemplate {
    IndexTemplate
}

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

impl WeatherDisplay {
    fn new(city: String, response: WeatherResponse) -> Self {
        let display = WeatherDisplay {
            city,
            forecasts: response
                .hourly
                .time
                .iter()
                .zip(response.hourly.temperature_2m.iter())
                .map(|(date, temperature)| Forecast {
                    date: date.to_string(),
                    temperature: temperature.to_string(),
                })
                .collect(),
        };
        display
    }
}

async fn get_lat_long(pool: &PgPool, name: &str) -> Result<LatLong, anyhow::Error> {
    let lat_long = sqlx::query_as::<_, LatLong>(
        "SELECT lat AS latitude, long AS longitude FROM cities WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await?;

    if let Some(lat_long) = lat_long {
        return Ok(lat_long);
    }

    let lat_long = fetch_lat_long(name).await?;
    sqlx::query("INSERT INTO cities (name, lat, long) VALUES ($1, $2, $3)")
        .bind(name)
        .bind(lat_long.latitude)
        .bind(lat_long.longitude)
        .execute(pool)
        .await?;

    Ok(lat_long)
}

async fn weather(
    Query(params): Query<WeatherQuery>,
    State(pool): State<PgPool>,
) -> Result<WeatherDisplay, AppError> {
    let lat_long = get_lat_long(&pool, &params.city).await?;
    let weather = fetch_weather(lat_long).await?;
    Ok(WeatherDisplay::new(params.city, weather))
}

/// A user that is authorized to access the stats endpoint.
///
/// No fields are required, we just need to know that the user is authorized. In
/// a production application you would probably want to have some kind of user
/// ID or similar here.
struct User;

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = axum::http::Response<axum::body::Body>;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|header| header.to_str().ok());

        if let Some(auth_header) = auth_header {
            if auth_header.starts_with("Basic ") {
                let credentials = auth_header.trim_start_matches("Basic ");
                let decoded = base64::decode(credentials).unwrap_or_default();
                let credential_str = from_utf8(&decoded).unwrap_or("");

                if credential_str == "forecast:forecast" {
                    return Ok(User);
                }
            }
        }

        let reject_response = axum::http::Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header(
                "WWW-Authenticate",
                "Basic realm=\"Please enter your credentials\"",
            )
            .body(axum::body::Body::from("Unauthorized"))
            .unwrap();

        Err(reject_response)
    }
}

#[derive(Template)]
#[template(path = "stats.html")]
struct StatsTemplate {
    pub cities: Vec<City>,
}

async fn stats(_user: User, State(pool): State<PgPool>) -> Result<StatsTemplate, AppError> {
    let cities = get_last_cities(&pool).await?;
    Ok(StatsTemplate { cities })
}

#[shuttle_runtime::main]
async fn main(#[shuttle_aws_rds::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats))
        .with_state(pool);

    Ok(router.into())
}
