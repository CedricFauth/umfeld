//! Raw JSON from the upstream APIs.

use serde::Deserialize;

use crate::response::{CurrentWeather, DailyWeather, FuelStation, HourlyWeather, Place, Weather};

#[derive(Debug, Deserialize)]
pub struct PlaceSearch {
    // missing when nothing was found
    #[serde(default)]
    pub results: Vec<GeocodedPlace>,
}

#[derive(Debug, Deserialize)]
pub struct GeocodedPlace {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub timezone: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub population: Option<u64>,
}

impl From<GeocodedPlace> for Place {
    fn from(place: GeocodedPlace) -> Self {
        Place {
            name: place.name,
            country: place.country,
            country_code: place.country_code,
            latitude: place.latitude,
            longitude: place.longitude,
            elevation: place.elevation,
            population: place.population,
            timezone: place.timezone,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Forecast {
    pub current: CurrentBlock,
    pub hourly: HourlyBlock,
    pub daily: DailyBlock,
}

impl From<Forecast> for Weather {
    fn from(forecast: Forecast) -> Self {
        Weather {
            current: CurrentWeather {
                time: forecast.current.time,
                temperature_2m: forecast.current.temperature_2m,
                apparent_temperature: forecast.current.apparent_temperature,
                precipitation: forecast.current.precipitation,
            },
            hourly: HourlyWeather {
                time: forecast.hourly.time,
                temperature_2m: forecast.hourly.temperature_2m,
                apparent_temperature: forecast.hourly.apparent_temperature,
                precipitation_probability: forecast.hourly.precipitation_probability,
                precipitation: forecast.hourly.precipitation,
            },
            daily: DailyWeather {
                time: forecast.daily.time,
                temperature_2m_max: forecast.daily.temperature_2m_max,
                temperature_2m_min: forecast.daily.temperature_2m_min,
                precipitation_probability_max: forecast.daily.precipitation_probability_max,
                precipitation_hours: forecast.daily.precipitation_hours,
                precipitation_sum: forecast.daily.precipitation_sum,
                uv_index_max: forecast.daily.uv_index_max,
                sunrise: forecast.daily.sunrise,
                sunset: forecast.daily.sunset,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CurrentBlock {
    pub time: String,
    pub temperature_2m: f64,
    pub apparent_temperature: f64,
    pub precipitation: f64,
}

#[derive(Debug, Deserialize)]
pub struct HourlyBlock {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
    pub apparent_temperature: Vec<f64>,
    pub precipitation_probability: Vec<u8>,
    pub precipitation: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DailyBlock {
    pub time: Vec<String>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_probability_max: Vec<u8>,
    pub precipitation_hours: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
    pub uv_index_max: Vec<f64>,
    pub sunrise: Vec<String>,
    pub sunset: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct FuelList {
    pub ok: bool,
    pub message: Option<String>,
    // missing in error responses
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub stations: Vec<Station>,
}

#[derive(Debug, Deserialize)]
pub struct Station {
    pub name: String,
    pub brand: String,
    pub street: String,
    pub place: String,
    pub lat: f64,
    pub lng: f64,
    pub dist: f64,
    pub diesel: Option<f64>,
    pub e5: Option<f64>,
    pub e10: Option<f64>,

    #[serde(rename = "isOpen")]
    pub is_open: bool,

    #[serde(rename = "houseNumber")]
    pub house_number: String,

    #[serde(rename = "postCode")]
    pub post_code: u32,
}

impl From<Station> for FuelStation {
    fn from(station: Station) -> Self {
        FuelStation {
            name: station.name,
            brand: station.brand,
            street: station.street,
            house_number: station.house_number,
            post_code: format!("{:05}", station.post_code),
            place: station.place,
            latitude: station.lat,
            longitude: station.lng,
            distance: station.dist,
            diesel: station.diesel,
            e5: station.e5,
            e10: station.e10,
            is_open: station.is_open,
        }
    }
}
