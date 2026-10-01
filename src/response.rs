//! Response types. Units: °C, mm, %, m, km, EUR/l.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub success: bool,
    pub error: Option<String>,
    pub datetime: String,
    pub attribution: Vec<Attribution>,
    pub place: Option<Place>,
    pub weather: Option<Weather>,
    pub fuel: Option<Vec<FuelStation>>,
}

#[derive(Debug, Serialize)]
pub struct Attribution {
    /// Response fields this applies to.
    pub covers: &'static [&'static str],
    pub source: &'static str,
    pub license: String,
    pub url: &'static str,
}

impl Attribution {
    pub fn open_meteo() -> Attribution {
        Attribution {
            covers: &["place", "weather"],
            source: "Open-Meteo",
            license: "CC BY 4.0".to_owned(),
            url: "https://open-meteo.com/",
        }
    }

    /// Uses the license text from the API if there is one.
    pub fn tankerkoenig(license: String) -> Attribution {
        Attribution {
            covers: &["fuel"],
            source: "Tankerkönig (MTS-K)",
            license: if license.is_empty() {
                "CC BY 4.0".to_owned()
            } else {
                license
            },
            url: "https://creativecommons.tankerkoenig.de/",
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Place {
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub timezone: String,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub population: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct Weather {
    pub current: CurrentWeather,
    pub hourly: HourlyWeather,
    pub daily: DailyWeather,
}

#[derive(Debug, Serialize)]
pub struct CurrentWeather {
    pub time: String,
    pub temperature_2m: f64,
    pub apparent_temperature: f64,
    pub precipitation: f64,
}

#[derive(Debug, Serialize)]
pub struct HourlyWeather {
    pub time: Vec<String>,
    pub temperature_2m: Vec<f64>,
    pub apparent_temperature: Vec<f64>,
    pub precipitation_probability: Vec<u8>,
    pub precipitation: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct DailyWeather {
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

#[derive(Debug, Serialize)]
pub struct FuelStation {
    pub name: String,
    pub brand: String,
    pub street: String,
    pub house_number: String,
    /// String to keep leading zeros (e.g. 01159).
    pub post_code: String,
    pub place: String,
    pub latitude: f64,
    pub longitude: f64,
    /// km
    pub distance: f64,
    pub diesel: Option<f64>,
    pub e5: Option<f64>,
    pub e10: Option<f64>,
    pub is_open: bool,
}
