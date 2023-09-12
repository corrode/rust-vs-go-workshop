use axum::{extract::Query, http::StatusCode, routing::get, Router};
use serde::Deserialize;
use std::net::SocketAddr;

#[derive(Deserialize, Debug)]
pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

#[derive(Deserialize, Debug, Clone)]
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

#[derive(Deserialize, Debug)]
pub struct WeatherDisplay {
    pub city: String,
    pub forecasts: Vec<Forecast>,
}

#[derive(Deserialize, Debug)]
pub struct Forecast {
    pub date: String,
    pub temperature: String,
}

async fn fetch_lat_long(city: &str) -> Result<LatLong, Box<dyn std::error::Error>> {
    let endpoint = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        city
    );
    let response = reqwest::get(&endpoint).await?.json::<GeoResponse>().await?;
    response
        .results
        .get(0)
        .cloned()
        .ok_or("No results found".into())
}

async fn fetch_weather(lat_long: LatLong) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
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

// basic handler that responds with a static string
async fn index() -> &'static str {
    "Index"
}

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

async fn weather(Query(params): Query<WeatherQuery>) -> Result<String, StatusCode> {
    let lat_long = fetch_lat_long(&params.city)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let weather = fetch_weather(lat_long)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let display = WeatherDisplay {
        city: params.city,
        forecasts: weather
            .hourly
            .time
            .iter()
            .zip(weather.hourly.temperature_2m.iter())
            .map(|(date, temperature)| Forecast {
                date: date.to_string(),
                temperature: temperature.to_string(),
            })
            .collect(),
    };
    Ok(format!("{:?}", display))
}

async fn stats() -> &'static str {
    "Stats"
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
