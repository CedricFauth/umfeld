//! Response types. Units: °C, mm, %, m, km, km/h, EUR/l.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct SnapshotResponse {
    pub success: bool,
    pub error: Option<String>,
    /// Unix seconds, when the response was generated.
    pub timestamp: i64,
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
    /// Unix seconds.
    pub time: i64,
    /// WMO weather interpretation code (0-99).
    pub weather_code: Option<u8>,
    pub condition: Option<Condition>,
    pub is_day: Option<bool>,
    pub temperature_2m: f64,
    pub apparent_temperature: f64,
    pub precipitation: f64,
    pub wind_speed_10m: Option<f64>,
    pub wind_gusts_10m: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct HourlyWeather {
    /// Unix seconds, start of each hour. Always 3600 apart, also across DST changes.
    pub time: Vec<i64>,
    /// WMO weather interpretation code (0-99).
    pub weather_code: Vec<Option<u8>>,
    pub condition: Vec<Option<Condition>>,
    /// Whether the sun is up during the hour.
    pub is_day: Vec<Option<bool>>,
    pub temperature_2m: Vec<f64>,
    pub apparent_temperature: Vec<f64>,
    pub precipitation_probability: Vec<u8>,
    pub precipitation: Vec<f64>,
    pub wind_speed_10m: Vec<f64>,
    pub wind_gusts_10m: Vec<f64>,
}

#[derive(Debug, Serialize)]
pub struct DailyWeather {
    /// Unix seconds, local midnight that starts each day. DST days are 23 or 25 h long.
    pub time: Vec<i64>,
    /// WMO weather interpretation code (0-99).
    pub weather_code: Vec<Option<u8>>,
    pub condition: Vec<Option<Condition>>,
    pub temperature_2m_max: Vec<f64>,
    pub temperature_2m_min: Vec<f64>,
    pub precipitation_probability_max: Vec<u8>,
    pub precipitation_hours: Vec<f64>,
    pub precipitation_sum: Vec<f64>,
    pub uv_index_max: Vec<f64>,
    pub wind_speed_10m_max: Vec<f64>,
    pub wind_gusts_10m_max: Vec<f64>,
    /// Unix seconds.
    pub sunrise: Vec<Option<i64>>,
    /// Unix seconds.
    pub sunset: Vec<Option<i64>>,
}

/// Coarse grouping of WMO codes. A closed set: the ESP draws unknown values as `overcast`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Condition {
    Clear,
    MostlyClear,
    PartlyCloudy,
    Overcast,
    Fog,
    Drizzle,
    Rain,
    Snow,
    Thunderstorm,
}

impl Condition {
    pub fn from_wmo_code(code: u8) -> Condition {
        match code {
            0 => Condition::Clear,
            1 => Condition::MostlyClear,
            2 => Condition::PartlyCloudy,
            45 | 48 => Condition::Fog,
            51 | 53 | 55 | 56 | 57 => Condition::Drizzle,
            61 | 63 | 65 | 66 | 67 | 80 | 81 | 82 => Condition::Rain,
            71 | 73 | 75 | 77 | 85 | 86 => Condition::Snow,
            95 | 96 | 99 => Condition::Thunderstorm,
            _ => Condition::Overcast,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct FuelStation {
    pub name: String,
    pub brand: String,
    /// `brand` (or `name`) without legal-form suffixes like GmbH.
    pub display_name: String,
    pub street: String,
    pub house_number: String,
    /// String to keep leading zeros (e.g. 01159).
    pub post_code: String,
    /// Title-cased if upstream sends it in all caps.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wmo_code_mapping() {
        let groups: [(&[u8], Condition); 9] = [
            (&[0], Condition::Clear),
            (&[1], Condition::MostlyClear),
            (&[2], Condition::PartlyCloudy),
            (&[3, 4, 50, 98, 255], Condition::Overcast),
            (&[45, 48], Condition::Fog),
            (&[51, 53, 55, 56, 57], Condition::Drizzle),
            (&[61, 63, 65, 66, 67, 80, 81, 82], Condition::Rain),
            (&[71, 73, 75, 77, 85, 86], Condition::Snow),
            (&[95, 96, 99], Condition::Thunderstorm),
        ];
        for (codes, condition) in groups {
            for &code in codes {
                assert_eq!(Condition::from_wmo_code(code), condition, "code {code}");
            }
        }
    }
}
