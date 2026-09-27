use reqwest::{Client, Error as ReqwestError};
use serde::Serialize;
use thiserror::Error;

use crate::{
    response::{FuelStation, Place, Weather},
    wire,
};

#[derive(Debug, Error)]
pub enum ApiProviderError {
    #[error("place was not found")]
    PlaceNotFound,

    #[error("fuel API error: {message}")]
    FuelApiError { message: String },

    #[error("request failed")]
    Request(#[from] ReqwestError),
}

#[derive(Serialize)]
struct WeatherQuery<'a> {
    latitude: f64,
    longitude: f64,
    daily: &'static str,
    hourly: &'static str,
    current: &'static str,
    timezone: &'a str,
    forecast_days: u8,
}

#[derive(Serialize)]
struct PlaceQuery<'a> {
    name: &'a str,
    count: u8,
    language: &'static str,
    format: &'static str,
}

pub(super) struct FuelPrices {
    pub stations: Vec<FuelStation>,
    pub license: String,
}

#[derive(Serialize)]
struct FuelPricesQuery<'a> {
    lat: f64,
    lng: f64,
    rad: f64,
    sort: &'static str,
    #[serde(rename = "type")]
    fuel_type: &'static str,
    apikey: &'a str,
}

pub(super) async fn resolve_place(client: &Client, name: &str) -> Result<Place, ApiProviderError> {
    let query = PlaceQuery {
        name,
        count: 1,
        language: "de",
        format: "json",
    };

    let response = match client
        .get("https://geocoding-api.open-meteo.com/v1/search")
        .query(&query)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(error) => {
            eprintln!("{error}");
            return Err(error.into());
        }
    };

    let place = response
        .json::<wire::PlaceSearch>()
        .await?
        .results
        .into_iter()
        .next()
        .ok_or(ApiProviderError::PlaceNotFound)?;

    Ok(place.into())
}

pub(super) async fn get_weather_data(
    client: &Client,
    place: &Place,
) -> Result<Weather, ApiProviderError> {
    let query = WeatherQuery {
        latitude: place.latitude,
        longitude: place.longitude,
        daily: "uv_index_max,sunrise,sunset",
        hourly: "temperature_2m,apparent_temperature,precipitation_probability,precipitation",
        current: "apparent_temperature,temperature_2m,precipitation",
        timezone: &place.timezone,
        forecast_days: 3,
    };

    let response = match client
        .get("https://api.open-meteo.com/v1/forecast")
        .query(&query)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(error) => {
            eprintln!("{error}");
            return Err(error.into());
        }
    };

    let weather = response.json::<wire::Forecast>().await?;

    Ok(weather.into())
}

pub(super) async fn get_fuel_prices(
    client: &Client,
    place: &Place,
    radius: f64,
    api_key: &str,
) -> Result<FuelPrices, ApiProviderError> {
    let query = FuelPricesQuery {
        lat: place.latitude,
        lng: place.longitude,
        rad: radius,
        sort: "dist",
        fuel_type: "all",
        apikey: api_key,
    };

    let response = match client
        .get("https://creativecommons.tankerkoenig.de/json/list.php")
        .query(&query)
        .send()
        .await
    {
        Ok(resp) => resp,
        Err(error) => {
            eprintln!("{error}");
            return Err(error.into());
        }
    };

    let fuel = response.json::<wire::FuelList>().await?;

    if !fuel.ok {
        let message = fuel
            .message
            .unwrap_or_else(|| "unknown fuel api error".to_owned());
        return Err(ApiProviderError::FuelApiError { message });
    }

    Ok(FuelPrices {
        stations: fuel.stations.into_iter().map(FuelStation::from).collect(),
        license: fuel.license,
    })
}
