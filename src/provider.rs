use chrono::{DateTime, Days, Offset, TimeDelta, Utc};
use chrono_tz::Tz;
use reqwest::{Client, Error as ReqwestError};
use serde::Serialize;
use thiserror::Error;

use crate::{
    local_time::{local_midnight, parse_tz},
    response::{FuelStation, Place, Weather},
    wire,
};

const FORECAST_DAYS: u64 = 3;
const FORECAST_HOURS: i64 = 72;

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
    timeformat: &'static str,
    start_date: String,
    end_date: String,
    start_hour: String,
    end_hour: String,
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
    let tz = parse_tz(&place.timezone);
    let window = ForecastWindow::starting_today(Utc::now().with_timezone(&tz));
    let query = WeatherQuery {
        latitude: place.latitude,
        longitude: place.longitude,
        daily: "weather_code,temperature_2m_max,temperature_2m_min,precipitation_probability_max,precipitation_hours,precipitation_sum,uv_index_max,wind_speed_10m_max,wind_gusts_10m_max,sunrise,sunset",
        hourly: "weather_code,is_day,temperature_2m,apparent_temperature,precipitation_probability,precipitation,wind_speed_10m,wind_gusts_10m",
        current: "weather_code,is_day,apparent_temperature,temperature_2m,precipitation,wind_speed_10m,wind_gusts_10m",
        timezone: &place.timezone,
        timeformat: "unixtime",
        start_date: window.start_date,
        end_date: window.end_date,
        start_hour: window.start_hour,
        end_hour: window.end_hour,
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

    Ok(weather.into_weather(&tz))
}

/// Dates and hours as Open-Meteo reads them: local, in the zone's offset at request time.
struct ForecastWindow {
    start_date: String,
    end_date: String,
    start_hour: String,
    end_hour: String,
}

impl ForecastWindow {
    /// Hourly data starts at today's real local midnight. On a DST-change day that differs
    /// from midnight in the current offset, which is what Open-Meteo would start at otherwise.
    fn starting_today(now: DateTime<Tz>) -> ForecastWindow {
        let today = now.date_naive();
        let midnight = local_midnight(&now.timezone(), today);
        let offset = now.offset().fix();
        let hour = |t: DateTime<Tz>| {
            t.with_timezone(&offset)
                .format("%Y-%m-%dT%H:%M")
                .to_string()
        };
        ForecastWindow {
            start_date: today.to_string(),
            end_date: (today + Days::new(FORECAST_DAYS - 1)).to_string(),
            start_hour: hour(midnight),
            end_hour: hour(midnight + TimeDelta::hours(FORECAST_HOURS - 1)),
        }
    }
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

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono_tz::Europe::Berlin;

    use super::*;

    fn window(y: i32, m: u32, d: u32, h: u32) -> ForecastWindow {
        ForecastWindow::starting_today(Berlin.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap())
    }

    #[test]
    fn normal_day() {
        let w = window(2026, 10, 3, 12);
        assert_eq!(w.start_date, "2026-10-03");
        assert_eq!(w.end_date, "2026-10-05");
        assert_eq!(w.start_hour, "2026-10-03T00:00");
        assert_eq!(w.end_hour, "2026-10-05T23:00");
    }

    #[test]
    fn end_of_dst() {
        // now is CET (+1), but midnight was CEST (+2): 22:00Z = 23:00 in +1
        let w = window(2026, 10, 25, 12);
        assert_eq!(w.start_date, "2026-10-25");
        assert_eq!(w.start_hour, "2026-10-24T23:00");
        assert_eq!(w.end_hour, "2026-10-27T22:00");
    }

    #[test]
    fn start_of_dst() {
        // now is CEST (+2), but midnight was CET (+1): 23:00Z = 01:00 in +2
        let w = window(2026, 3, 29, 12);
        assert_eq!(w.start_hour, "2026-03-29T01:00");
    }
}
