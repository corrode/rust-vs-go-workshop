use axum::{
    extract::{Path, Query},
    routing::get,
    Router,
};
use serde::Deserialize;
use std::{collections::HashMap, net::SocketAddr};

#[derive(Deserialize, Debug)]
pub struct GeoResponse {
    pub results: Vec<LatLong>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LatLong {
    pub latitude: f64,
    pub longitude: f64,
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

// basic handler that responds with a static string
async fn index() -> &'static str {
    "Index"
}

#[derive(Deserialize)]
pub struct WeatherQuery {
    pub city: String,
}

async fn weather(Query(params): Query<WeatherQuery>) -> String {
    let lat_long = fetch_lat_long(&params.city).await.unwrap();
    format!(
        "{}: {}, {}",
        params.city, lat_long.latitude, lat_long.longitude
    )
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
