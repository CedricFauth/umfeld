//! Raw JSON from the upstream APIs.

use chrono::DateTime;
use chrono_tz::Tz;
use serde::Deserialize;

use crate::{
    fuel_text,
    local_time::local_midnight,
    response::{
        Condition, CurrentWeather, DailyWeather, FuelStation, HourlyWeather, Place, Weather,
    },
};

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
    /// The one offset Open-Meteo applies to the whole response, even across a DST change.
    pub utc_offset_seconds: i64,
    pub current: CurrentBlock,
    pub hourly: HourlyBlock,
    pub daily: DailyBlock,
}

impl Forecast {
    /// Expects `timeformat=unixtime`. `tz` is the zone the forecast was requested for.
    pub fn into_weather(self, tz: &Tz) -> Weather {
        let offset = self.utc_offset_seconds;
        Weather {
            current: CurrentWeather {
                time: self.current.time,
                weather_code: self.current.weather_code,
                condition: self.current.weather_code.map(Condition::from_wmo_code),
                is_day: self.current.is_day.map(|d| d != 0),
                temperature_2m: self.current.temperature_2m,
                apparent_temperature: self.current.apparent_temperature,
                precipitation: self.current.precipitation,
                wind_speed_10m: self.current.wind_speed_10m,
                wind_gusts_10m: self.current.wind_gusts_10m,
            },
            hourly: HourlyWeather {
                time: self.hourly.time,
                condition: conditions(&self.hourly.weather_code),
                weather_code: self.hourly.weather_code,
                is_day: self
                    .hourly
                    .is_day
                    .iter()
                    .map(|d| d.map(|d| d != 0))
                    .collect(),
                temperature_2m: self.hourly.temperature_2m,
                apparent_temperature: self.hourly.apparent_temperature,
                precipitation_probability: self.hourly.precipitation_probability,
                precipitation: self.hourly.precipitation,
                wind_speed_10m: self.hourly.wind_speed_10m,
                wind_gusts_10m: self.hourly.wind_gusts_10m,
            },
            daily: DailyWeather {
                time: self
                    .daily
                    .time
                    .iter()
                    .map(|&t| day_start(t, offset, tz))
                    .collect(),
                condition: conditions(&self.daily.weather_code),
                weather_code: self.daily.weather_code,
                temperature_2m_max: self.daily.temperature_2m_max,
                temperature_2m_min: self.daily.temperature_2m_min,
                precipitation_probability_max: self.daily.precipitation_probability_max,
                precipitation_hours: self.daily.precipitation_hours,
                precipitation_sum: self.daily.precipitation_sum,
                uv_index_max: self.daily.uv_index_max,
                wind_speed_10m_max: self.daily.wind_speed_10m_max,
                wind_gusts_10m_max: self.daily.wind_gusts_10m_max,
                sunrise: self.daily.sunrise,
                sunset: self.daily.sunset,
            },
        }
    }
}

fn conditions(codes: &[Option<u8>]) -> Vec<Option<Condition>> {
    codes
        .iter()
        .map(|code| code.map(Condition::from_wmo_code))
        .collect()
}

/// Open-Meteo's daily time is midnight in its single response offset, which is an hour
/// off on days with a different offset. Recover the date and use the real local midnight.
fn day_start(upstream: i64, offset: i64, tz: &Tz) -> i64 {
    match DateTime::from_timestamp(upstream + offset, 0) {
        Some(shifted) => local_midnight(tz, shifted.date_naive()).timestamp(),
        None => upstream,
    }
}

// Times are Unix seconds (timeformat=unixtime). Values are null where Open-Meteo has none.

#[derive(Debug, Deserialize)]
pub struct CurrentBlock {
    pub time: i64,
    pub weather_code: Option<u8>,
    pub is_day: Option<u8>,
    pub temperature_2m: f64,
    pub apparent_temperature: f64,
    pub precipitation: f64,
    pub wind_speed_10m: Option<f64>,
    pub wind_gusts_10m: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct HourlyBlock {
    pub time: Vec<i64>,
    pub weather_code: Vec<Option<u8>>,
    pub is_day: Vec<Option<u8>>,
    pub temperature_2m: Vec<f64>,
    pub apparent_temperature: Vec<f64>,
    pub precipitation_probability: Vec<u8>,
    pub precipitation: Vec<f64>,
    pub wind_speed_10m: Vec<f64>,
    pub wind_gusts_10m: Vec<f64>,
}

#[derive(Debug, Deserialize)]
pub struct DailyBlock {
    pub time: Vec<i64>,
    pub weather_code: Vec<Option<u8>>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_probability_max: Vec<u8>,
    pub precipitation_hours: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
    pub uv_index_max: Vec<f64>,
    pub wind_speed_10m_max: Vec<f64>,
    pub wind_gusts_10m_max: Vec<f64>,
    pub sunrise: Vec<Option<i64>>,
    pub sunset: Vec<Option<i64>>,
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
    pub brand: Option<String>,
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
        let brand = station.brand.unwrap_or_default();
        FuelStation {
            display_name: fuel_text::display_name(&brand, &station.name),
            name: station.name,
            brand,
            street: station.street,
            house_number: station.house_number,
            post_code: format!("{:05}", station.post_code),
            place: fuel_text::place(&station.place),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Trimmed real response for Sydney, 2026-10-03 to 05, requested on 2026-10-09 (AEDT,
    /// +11). Clocks went from +10 to +11 on 2026-10-04 at 02:00.
    const SYDNEY_DST: &str = r#"{
        "utc_offset_seconds": 39600,
        "current": { "time": 1791576000, "weather_code": 3, "is_day": 1,
                     "temperature_2m": 18.0, "apparent_temperature": 17.0, "precipitation": 0.0,
                     "wind_speed_10m": 13.2, "wind_gusts_10m": 23.4 },
        "hourly": { "time": [1790949600, 1790953200, 1790956800],
                    "weather_code": [95, null, 1], "is_day": [0, 1, null],
                    "temperature_2m": [1.0, 2.0, 3.0], "apparent_temperature": [1.0, 2.0, 3.0],
                    "precipitation_probability": [0, 0, 0], "precipitation": [0.0, 0.0, 0.0],
                    "wind_speed_10m": [1.0, 2.0, 3.0], "wind_gusts_10m": [1.0, 2.0, 3.0] },
        "daily": { "time": [1790946000, 1791032400, 1791118800],
                   "weather_code": [95, 51, 200],
                   "temperature_2m_max": [1.0, 2.0, 3.0], "temperature_2m_min": [1.0, 2.0, 3.0],
                   "precipitation_probability_max": [0, 0, 0],
                   "precipitation_hours": [0.0, 0.0, 0.0], "precipitation_sum": [0.0, 0.0, 0.0],
                   "uv_index_max": [1.0, 2.0, 3.0], "wind_speed_10m_max": [1.0, 2.0, 3.0],
                   "wind_gusts_10m_max": [1.0, 2.0, 3.0],
                   "sunrise": [1790969408, 1791055726, null],
                   "sunset": [1791014346, 1791100790, 1791187234] }
    }"#;

    fn sydney() -> Weather {
        serde_json::from_str::<Forecast>(SYDNEY_DST)
            .unwrap()
            .into_weather(&chrono_tz::Australia::Sydney)
    }

    #[test]
    fn daily_time_is_real_local_midnight() {
        let daily = sydney().daily;
        // 00:00 AEST (+10), 00:00 AEST, 00:00 AEDT (+11)
        assert_eq!(daily.time, [1790949600, 1791036000, 1791118800]);
        assert_eq!(daily.time[2] - daily.time[1], 23 * 3600);
        assert_eq!(daily.sunrise, [Some(1790969408), Some(1791055726), None]);
    }

    #[test]
    fn conditions_and_is_day() {
        let weather = sydney();
        assert_eq!(weather.current.condition, Some(Condition::Overcast));
        assert_eq!(weather.current.is_day, Some(true));
        assert_eq!(
            weather.hourly.condition,
            [
                Some(Condition::Thunderstorm),
                None,
                Some(Condition::MostlyClear)
            ]
        );
        assert_eq!(weather.hourly.is_day, [Some(false), Some(true), None]);
        assert_eq!(weather.daily.condition[2], Some(Condition::Overcast));
    }
}
